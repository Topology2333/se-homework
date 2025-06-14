use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::models::{ChargingMode, ChargingPile, ChargingRequest};
use crate::scheduler::queue_manager::{QueueManager, PileInfo};
use std::collections::HashMap;
use chrono::Utc;

/// 调度器
pub struct Dispatcher {
    queue_manager: Arc<QueueManager>,
    is_calling: RwLock<bool>,
}

impl Dispatcher {
    pub fn new(queue_manager: Arc<QueueManager>) -> Self {
        Self {
            queue_manager,
            is_calling: RwLock::new(false),
        }
    }

    /// 启动叫号服务
    pub async fn start_calling(&self) {
        let mut is_calling = self.is_calling.write().await;
        *is_calling = true;
        println!("叫号服务已启动");
    }

    /// 停止叫号服务
    pub async fn stop_calling(&self) {
        let mut is_calling = self.is_calling.write().await;
        *is_calling = false;
        println!("叫号服务已停止");
    }

    /// 检查叫号服务是否在运行
    pub async fn is_calling(&self) -> bool {
        *self.is_calling.read().await
    }

    /// 系统tick
    pub async fn tick(&self) {
        // 检查充电完成并启动下一辆车
        self.queue_manager.tick().await;
        
        // 如果叫号服务在运行，则调度等候车辆
        if self.is_calling().await {
            self.dispatch_waiting_vehicles().await;
        }
    }

    /// 调度等候车辆
    async fn dispatch_waiting_vehicles(&self) {
        let mut pile_infos = self.queue_manager.pile_infos.write().await;
        let mut waiting_queue = self.queue_manager.waiting_queue.write().await;
        let now = self.queue_manager.time_system.current_time();

        // 收集所有等候区请求（克隆，避免借用冲突）
        let requests_to_dispatch: Vec<_> = waiting_queue.iter().cloned().collect();

        for request in requests_to_dispatch {
            // 找到所有同类型且队列未满的充电桩
            let mut best_pile: Option<(String, f64)> = None;
            for (pile_number, pile_info) in pile_infos.iter() {
                let pile = pile_info.pile.read().await;
                if pile.mode.to_string() == request.mode && pile_info.queue.len() < 1 {
                    // 计算完成时间
                    let completion_time = pile_info.calculate_completion_time(&request, &self.queue_manager.time_system).await;
                    if best_pile.is_none() || completion_time < best_pile.as_ref().unwrap().1 {
                        best_pile = Some((pile_number.clone(), completion_time));
                    }
                }
            }
            if let Some((best_pile_number, _)) = best_pile {
                if let Some(pile_info) = pile_infos.get_mut(&best_pile_number) {
                    // 从等候区移除
                    if let Some(idx) = waiting_queue.iter().position(|r| r.id == request.id) {
                        let request_arc = waiting_queue.remove(idx).unwrap();
                        pile_info.queue.push_back(request_arc.clone());
                        println!("✅ 用户 {} 已加入充电桩 {} 队列", request_arc.user_id, best_pile_number);
                        // 立即开始充电（如果当前没人充电）
                        pile_info.start_next_charging(now).await;
                    }
                }
            }
        }
    }

    /// 找到最佳充电桩
    async fn find_best_pile(
        &self,
        request: &ChargingRequest,
        pile_infos: &HashMap<String, PileInfo>,
    ) -> Option<(String, f64)> {
        let mut best_pile = None;
        let mut min_completion_time = f64::MAX;

        for (pile_number, pile_info) in pile_infos {
            // 检查充电桩是否可用、模式匹配，并且队列未满
            let pile = pile_info.pile.read().await;
            if pile.mode == request.mode.parse().unwrap() && pile_info.has_space() {
                // 计算完成时间
                let completion_time = pile_info.calculate_completion_time(request, &self.queue_manager.time_system).await;
                
                // 更新最佳充电桩
                if completion_time < min_completion_time {
                    min_completion_time = completion_time;
                    best_pile = Some((pile_number.clone(), completion_time));
                }
            }
        }

        best_pile
    }

    /// 处理充电桩故障
    pub async fn handle_pile_fault(&self, pile_id: &str) {
        // 更新充电桩状态
        if let Some(pile_info) = self.queue_manager.pile_infos.read().await.get(pile_id) {
            let mut pile = pile_info.pile.blocking_write();
            pile.report_fault();
        }
    }

    /// 处理充电桩恢复
    pub async fn handle_pile_recovery(&self, pile_id: &str) {
        // 更新充电桩状态
        if let Some(pile_info) = self.queue_manager.pile_infos.read().await.get(pile_id) {
            let mut pile = pile_info.pile.blocking_write();
            pile.repair().unwrap_or_else(|e| println!("修复充电桩失败: {}", e));
        }
    }
}