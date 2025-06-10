use super::queue_manager::QueueManager;
use crate::models::{
    ChargingMode, ChargingPile, ChargingRequest, FAST_CHARGING_POWER, SLOW_CHARGING_POWER,
};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 调度器
pub struct Dispatcher {
    queue_manager: Arc<QueueManager>,
    is_calling: RwLock<bool>, // 是否正在叫号
}

impl Dispatcher {
    pub fn new(queue_manager: Arc<QueueManager>) -> Self {
        Self {
            queue_manager,
            is_calling: RwLock::new(false),
        }
    }

    /// 为充电桩分配下一个车辆
    pub async fn dispatch_next_vehicle(&self, pile: &ChargingPile) -> Result<(), String> {
        // 获取等候区中对应模式的等待数量
        let waiting_count = self.queue_manager.get_waiting_count(pile.mode).await;
        if waiting_count == 0 {
            return Ok(());
        }

        // 检查充电桩队列是否有空位
        let queue_length = self.queue_manager.get_pile_queue_length(&pile.number).await;
        if queue_length >= 2 {
            // 每个充电桩队列最多2辆车
            return Ok(());
        }

        // 获取等候区中的所有请求，找到对应模式的第一个请求
        let waiting_requests = self.queue_manager.get_waiting_queue().await;
        if let Some(request) = waiting_requests
            .iter()
            .find(|r| r.mode == pile.mode.to_string())
        {
            // 将请求从等候区移除
            if let Some(request) = self
                .queue_manager
                .remove_from_waiting_queue(request.id)
                .await
            {
                // 添加到充电桩队列
                self.queue_manager
                    .add_to_pile_queue(&pile.number, request)
                    .await?;
            }
        }

        Ok(())
    }

    // 计算在指定充电桩完成充电所需的总时间
    async fn calculate_total_time(&self, pile: &ChargingPile, request: &ChargingRequest) -> f64 {
        let queue_length = self.queue_manager.get_pile_queue_length(&pile.number).await;
        let power = match pile.mode {
            ChargingMode::Fast => FAST_CHARGING_POWER,
            ChargingMode::Slow => SLOW_CHARGING_POWER,
        };

        // 计算队列中所有车辆的充电时间
        let mut total_time = 0.0;
        if queue_length > 0 {
            // 这里简化处理，假设队列中每辆车的充电量都是30度
            total_time = (queue_length as f64) * (30.0 / power);
        }

        // 加上当前请求的充电时间
        total_time + (request.amount / power)
    }

    // 为请求选择最佳充电桩
    async fn select_best_pile(&self, request: &ChargingRequest) -> Option<Arc<ChargingPile>> {
        let mode = ChargingMode::from(request.mode.clone());
        let available_piles = self.queue_manager.get_available_piles(mode).await;

        if available_piles.is_empty() {
            return None;
        }

        let mut best_pile = None;
        let mut min_time = f64::MAX;

        for pile in available_piles {
            let total_time = self.calculate_total_time(&pile, request).await;
            if total_time < min_time {
                min_time = total_time;
                best_pile = Some(pile.clone());
            }
        }

        best_pile
    }

    // 开始叫号服务
    pub async fn start_calling(&self) -> Result<(), String> {
        let mut is_calling = self.is_calling.write().await;
        if *is_calling {
            return Ok(());
        }
        *is_calling = true;
        Ok(())
    }

    // 停止叫号服务
    pub async fn stop_calling(&self) -> Result<(), String> {
        let mut is_calling = self.is_calling.write().await;
        *is_calling = false;
        Ok(())
    }

    // 处理充电请求
    pub async fn handle_request(&self, request: Arc<ChargingRequest>) -> Result<(), String> {
        // 检查是否在叫号
        if !*self.is_calling.read().await {
            return Err("叫号服务未启动".to_string());
        }

        // 选择最佳充电桩
        let best_pile = self
            .select_best_pile(&request)
            .await
            .ok_or("没有可用的充电桩".to_string())?;

        // 将请求添加到充电桩队列
        self.queue_manager
            .add_to_pile_queue(&best_pile.number, request)
            .await
    }

    // 处理充电桩故障
    pub async fn handle_pile_fault(&self, pile_id: &str) -> Result<(), String> {
        // 停止叫号服务
        self.stop_calling().await?;

        // 获取故障充电桩的信息
        let piles = self.queue_manager.get_piles().await;
        let _pile = piles.get(pile_id).ok_or("充电桩不存在".to_string())?;

        // 获取故障充电桩的队列
        let requests = self
            .queue_manager
            .get_pile_queue(pile_id)
            .await
            .ok_or("无法获取充电桩队列".to_string())?;

        // 将故障队列中的车辆重新分配到其他充电桩
        for request in requests {
            // 选择最佳充电桩（排除故障充电桩）
            if let Some(best_pile) = self.select_best_pile(&request).await {
                // 将请求添加到新的充电桩队列
                self.queue_manager
                    .add_to_pile_queue(&best_pile.number, request)
                    .await?;
            } else {
                // 如果没有可用的充电桩，将请求放回等候区
                self.queue_manager.add_to_waiting_queue(request).await?;
            }
        }

        // 清空故障充电桩的队列
        self.queue_manager.clear_pile_queue(pile_id).await?;

        // 重启叫号服务
        self.start_calling().await?;

        Ok(())
    }

    // 处理充电桩恢复
    pub async fn handle_pile_recovery(&self, pile_id: &str) -> Result<(), String> {
        // 停止叫号服务
        self.stop_calling().await?;

        // 获取所有同类型充电桩中尚未充电的车辆
        let piles = self.queue_manager.get_piles().await;
        let recovered_pile = piles.get(pile_id).ok_or("充电桩不存在".to_string())?;

        let mut all_requests = Vec::new();

        // 收集所有同类型充电桩中的请求
        for (pid, pile) in piles.iter() {
            if pile.mode == recovered_pile.mode && pid != pile_id {
                if let Some(requests) = self.queue_manager.get_pile_queue(pid).await {
                    all_requests.extend(requests);
                }
            }
        }

        // 按照排队号码排序
        all_requests.sort_by(|a, b| a.queue_number.cmp(&b.queue_number));

        // 清空所有相关充电桩的队列
        for (pid, pile) in piles.iter() {
            if pile.mode == recovered_pile.mode {
                self.queue_manager.clear_pile_queue(pid).await?;
            }
        }

        // 重新分配所有请求
        for request in all_requests {
            if let Some(best_pile) = self.select_best_pile(&request).await {
                self.queue_manager
                    .add_to_pile_queue(&best_pile.number, request)
                    .await?;
            } else {
                // 如果没有可用的充电桩，将请求放回等候区
                self.queue_manager.add_to_waiting_queue(request).await?;
            }
        }

        // 重启叫号服务
        self.start_calling().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dispatcher() {
        let queue_manager = Arc::new(QueueManager::new());
        let dispatcher = Dispatcher::new(queue_manager.clone());

        // 添加测试充电桩
        let pile1 = Arc::new(ChargingPile::new("A1".to_string(), ChargingMode::Fast));
        let pile2 = Arc::new(ChargingPile::new("A2".to_string(), ChargingMode::Fast));
        queue_manager.add_pile(pile1).await;
        queue_manager.add_pile(pile2).await;

        // 启动叫号服务
        assert!(dispatcher.start_calling().await.is_ok());

        // 创建测试请求
        let request = Arc::new(ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        ));

        // 测试请求处理
        assert!(dispatcher.handle_request(request).await.is_ok());

        // 停止叫号服务
        assert!(dispatcher.stop_calling().await.is_ok());
    }
}
