use crate::models::{ChargingMode, ChargingPile, ChargingRequest};
use super::QueueManager;
use std::sync::Arc;

/// 调度器
pub struct Dispatcher {
    queue_manager: Arc<QueueManager>,
}

impl Dispatcher {
    pub fn new(queue_manager: Arc<QueueManager>) -> Self {
        Self { queue_manager }
    }

    /// 为充电桩分配下一个车辆
    pub fn dispatch_next_vehicle(&self, pile: &ChargingPile) -> Result<Option<ChargingRequest>, String> {
        // 根据充电桩类型获取对应模式的下一个请求
        if let Some(request) = self.queue_manager.get_next_request(pile.mode) {
            Ok(Some(request))
        } else {
            Ok(None)
        }
    }
} 