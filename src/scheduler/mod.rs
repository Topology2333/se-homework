pub mod queue_manager;
pub mod dispatcher;
mod number_generator;

pub use queue_manager::QueueManager;
pub use dispatcher::Dispatcher;
pub use number_generator::QueueNumberGenerator;

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use uuid::Uuid;
use crate::models::{ChargingMode, ChargingPile, ChargingRequest};

/// 调度系统
pub struct ChargingScheduler {
    queue_manager: Arc<QueueManager>,
    number_generator: Arc<QueueNumberGenerator>,
    dispatcher: Arc<Dispatcher>,
}

impl ChargingScheduler {
    pub fn new() -> Self {
        let queue_manager = Arc::new(QueueManager::new());
        let number_generator = Arc::new(QueueNumberGenerator::new());
        let dispatcher = Arc::new(Dispatcher::new(queue_manager.clone()));

        Self {
            queue_manager,
            number_generator,
            dispatcher,
        }
    }

    /// 提交充电请求
    pub async fn submit_request(
        &self,
        user_id: Uuid,
        mode: ChargingMode,
        amount: f64,
    ) -> Result<ChargingRequest, String> {
        // 生成排队号码
        let queue_number = self.number_generator.generate(mode);
        
        // 创建充电请求
        let request = Arc::new(ChargingRequest::new(
            user_id,
            mode,
            amount,
            queue_number,
        ));

        // 添加到等候区
        self.queue_manager.add_to_waiting_queue(request.clone()).await?;

        Ok((*request).clone())
    }

    /// 取消充电请求
    pub async fn cancel_request(&self, request: &ChargingRequest) -> Result<(), String> {
        // 从等候区移除请求
        if let Some(_) = self.queue_manager.remove_from_waiting_queue(request.id).await {
            Ok(())
        } else {
            Err("请求不存在".to_string())
        }
    }

    /// 获取等候区请求列表
    pub async fn get_waiting_requests(&self) -> Vec<Arc<ChargingRequest>> {
        self.queue_manager.get_waiting_queue().await
    }

    /// 获取等候区中指定模式的等待数量
    pub async fn get_waiting_count(&self, mode: ChargingMode) -> usize {
        self.queue_manager.get_waiting_count(mode).await
    }

    /// 处理充电桩可用
    pub async fn handle_pile_available(&self, pile: &ChargingPile) -> Result<(), String> {
        self.dispatcher.dispatch_next_vehicle(pile).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_submit_request() {
        let scheduler = ChargingScheduler::new();
        
        let result = scheduler.submit_request(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
        ).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.amount, 30.0);
        assert_eq!(request.mode, ChargingMode::Fast);
        assert!(request.queue_number.starts_with('F'));
    }

    #[tokio::test]
    async fn test_cancel_request() {
        let scheduler = ChargingScheduler::new();
        
        // 提交请求
        let request = scheduler.submit_request(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
        ).await.unwrap();

        // 取消请求
        assert!(scheduler.cancel_request(&request).await.is_ok());
    }

    #[tokio::test]
    async fn test_waiting_count() {
        let scheduler = ChargingScheduler::new();
        
        // 提交两个快充请求
        scheduler.submit_request(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
        ).await.unwrap();
        
        scheduler.submit_request(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
        ).await.unwrap();

        // 提交一个慢充请求
        scheduler.submit_request(
            Uuid::new_v4(),
            ChargingMode::Slow,
            30.0,
        ).await.unwrap();

        assert_eq!(scheduler.get_waiting_count(ChargingMode::Fast).await, 2);
        assert_eq!(scheduler.get_waiting_count(ChargingMode::Slow).await, 1);
    }
} 