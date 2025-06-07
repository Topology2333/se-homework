mod queue_manager;
mod number_generator;
mod dispatcher;

pub use queue_manager::*;
pub use number_generator::*;
pub use dispatcher::*;

use crate::models::{ChargingMode, ChargingRequest, ChargingPile};
use parking_lot::RwLock;
use std::sync::Arc;

/// 调度系统
pub struct ChargingScheduler {
    queue_manager: Arc<QueueManager>,
    number_generator: Arc<NumberGenerator>,
    dispatcher: Arc<Dispatcher>,
}

impl ChargingScheduler {
    pub fn new() -> Self {
        let queue_manager = Arc::new(QueueManager::new());
        let number_generator = Arc::new(NumberGenerator::new());
        let dispatcher = Arc::new(Dispatcher::new(queue_manager.clone()));

        Self {
            queue_manager,
            number_generator,
            dispatcher,
        }
    }

    /// 提交充电请求
    pub fn submit_request(&self, user_id: uuid::Uuid, mode: ChargingMode, amount: f64) -> Result<ChargingRequest, String> {
        // 生成排队号码
        let queue_number = self.number_generator.generate_number(mode)?;
        
        // 创建充电请求
        let request = ChargingRequest::new(user_id, mode, amount, queue_number);
        
        // 加入等候区
        self.queue_manager.add_to_waiting_area(request.clone())?;
        
        Ok(request)
    }

    /// 处理充电桩空位
    pub fn handle_pile_available(&self, pile: &ChargingPile) -> Result<Option<ChargingRequest>, String> {
        self.dispatcher.dispatch_next_vehicle(pile)
    }

    /// 取消充电请求
    pub fn cancel_request(&self, request: &ChargingRequest) -> Result<(), String> {
        self.queue_manager.remove_request(request)
    }

    /// 获取等候区状态
    pub fn get_waiting_area_status(&self) -> Vec<ChargingRequest> {
        self.queue_manager.get_waiting_requests()
    }

    /// 获取指定模式的等待车辆数量
    pub fn get_waiting_count(&self, mode: ChargingMode) -> usize {
        self.queue_manager.get_waiting_count(mode)
    }
} 