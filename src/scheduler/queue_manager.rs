use crate::models::{ChargingMode, ChargingRequest, RequestStatus, WAITING_AREA_CAPACITY};
use parking_lot::RwLock;
use std::collections::VecDeque;

/// 等候区队列管理器
pub struct QueueManager {
    // 等候区队列
    waiting_area: RwLock<VecDeque<ChargingRequest>>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            waiting_area: RwLock::new(VecDeque::new()),
        }
    }

    /// 添加请求到等候区
    pub fn add_to_waiting_area(&self, request: ChargingRequest) -> Result<(), String> {
        let mut waiting_area = self.waiting_area.write();
        
        if waiting_area.len() >= WAITING_AREA_CAPACITY {
            return Err("等候区已满".to_string());
        }
        
        waiting_area.push_back(request);
        Ok(())
    }

    /// 从等候区移除请求
    pub fn remove_request(&self, request: &ChargingRequest) -> Result<(), String> {
        let mut waiting_area = self.waiting_area.write();
        
        if let Some(pos) = waiting_area.iter().position(|r| r.id == request.id) {
            waiting_area.remove(pos);
            Ok(())
        } else {
            Err("请求不在等候区".to_string())
        }
    }

    /// 获取指定模式下一个等待的请求
    pub fn get_next_request(&self, mode: ChargingMode) -> Option<ChargingRequest> {
        let mut waiting_area = self.waiting_area.write();
        
        let pos = waiting_area.iter().position(|r| 
            r.mode == mode && r.status == RequestStatus::Waiting
        );
        
        pos.map(|i| waiting_area.remove(i).unwrap())
    }

    /// 获取所有等待中的请求
    pub fn get_waiting_requests(&self) -> Vec<ChargingRequest> {
        self.waiting_area.read().iter().cloned().collect()
    }

    /// 获取指定模式的等待数量
    pub fn get_waiting_count(&self, mode: ChargingMode) -> usize {
        self.waiting_area.read()
            .iter()
            .filter(|r| r.mode == mode && r.status == RequestStatus::Waiting)
            .count()
    }

    /// 更新请求状态
    pub fn update_request_status(&self, request_id: uuid::Uuid, new_status: RequestStatus) -> Result<(), String> {
        let mut waiting_area = self.waiting_area.write();
        
        if let Some(request) = waiting_area.iter_mut().find(|r| r.id == request_id) {
            match (request.status, new_status) {
                (RequestStatus::Waiting, RequestStatus::Charging) |
                (RequestStatus::Waiting, RequestStatus::Cancelled) |
                (RequestStatus::Charging, RequestStatus::Completed) |
                (RequestStatus::Charging, RequestStatus::Cancelled) => {
                    request.status = new_status;
                    Ok(())
                },
                _ => Err("无效的状态转换".to_string())
            }
        } else {
            Err("请求不存在".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_add_and_remove_request() {
        let manager = QueueManager::new();
        let request = ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        );

        // 添加请求
        assert!(manager.add_to_waiting_area(request.clone()).is_ok());
        assert_eq!(manager.get_waiting_count(ChargingMode::Fast), 1);

        // 移除请求
        assert!(manager.remove_request(&request).is_ok());
        assert_eq!(manager.get_waiting_count(ChargingMode::Fast), 0);
    }

    #[test]
    fn test_waiting_area_capacity() {
        let manager = QueueManager::new();
        
        // 添加最大容量的请求
        for i in 0..WAITING_AREA_CAPACITY {
            let request = ChargingRequest::new(
                Uuid::new_v4(),
                ChargingMode::Fast,
                30.0,
                format!("F{}", i + 1),
            );
            assert!(manager.add_to_waiting_area(request).is_ok());
        }

        // 尝试添加超出容量的请求
        let extra_request = ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
            format!("F{}", WAITING_AREA_CAPACITY + 1),
        );
        assert!(manager.add_to_waiting_area(extra_request).is_err());
    }

    #[test]
    fn test_get_next_request() {
        let manager = QueueManager::new();
        
        // 添加一个快充请求和一个慢充请求
        let fast_request = ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        );
        let slow_request = ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Slow,
            30.0,
            "T1".to_string(),
        );

        manager.add_to_waiting_area(fast_request.clone()).unwrap();
        manager.add_to_waiting_area(slow_request).unwrap();

        // 获取下一个快充请求
        let next = manager.get_next_request(ChargingMode::Fast).unwrap();
        assert_eq!(next.id, fast_request.id);
        assert_eq!(manager.get_waiting_count(ChargingMode::Fast), 0);
    }

    #[test]
    fn test_status_transitions() {
        let manager = QueueManager::new();
        let request = ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        );
        
        // 添加请求
        manager.add_to_waiting_area(request.clone()).unwrap();
        
        // 等待 -> 充电
        assert!(manager.update_request_status(request.id, RequestStatus::Charging).is_ok());
        
        // 充电 -> 完成
        assert!(manager.update_request_status(request.id, RequestStatus::Completed).is_ok());
        
        // 完成 -> 取消（无效转换）
        assert!(manager.update_request_status(request.id, RequestStatus::Cancelled).is_err());
    }

    #[test]
    fn test_concurrent_mode_requests() {
        let manager = QueueManager::new();
        
        // 添加交替的快充和慢充请求
        for i in 0..4 {
            let mode = if i % 2 == 0 { 
                ChargingMode::Fast 
            } else { 
                ChargingMode::Slow 
            };
            
            let request = ChargingRequest::new(
                Uuid::new_v4(),
                mode,
                30.0,
                format!("{}{}", if mode == ChargingMode::Fast { "F" } else { "T" }, i/2 + 1),
            );
            manager.add_to_waiting_area(request).unwrap();
        }
        
        // 验证计数
        assert_eq!(manager.get_waiting_count(ChargingMode::Fast), 2);
        assert_eq!(manager.get_waiting_count(ChargingMode::Slow), 2);
        
        // 验证获取顺序
        let first_fast = manager.get_next_request(ChargingMode::Fast).unwrap();
        assert_eq!(first_fast.queue_number, "F1");
        
        let first_slow = manager.get_next_request(ChargingMode::Slow).unwrap();
        assert_eq!(first_slow.queue_number, "T1");
    }

    #[test]
    fn test_request_not_found() {
        let manager = QueueManager::new();
        let request = ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        );
        
        // 尝试移除不存在的请求
        assert!(manager.remove_request(&request).is_err());
        
        // 尝试更新不存在的请求状态
        assert!(manager.update_request_status(request.id, RequestStatus::Charging).is_err());
    }

    #[test]
    fn test_waiting_requests_order() {
        let manager = QueueManager::new();
        
        // 添加三个请求
        for i in 1..=3 {
            let request = ChargingRequest::new(
                Uuid::new_v4(),
                ChargingMode::Fast,
                30.0,
                format!("F{}", i),
            );
            manager.add_to_waiting_area(request).unwrap();
        }
        
        // 验证返回的等待请求列表顺序
        let waiting_requests = manager.get_waiting_requests();
        assert_eq!(waiting_requests.len(), 3);
        for (i, request) in waiting_requests.iter().enumerate() {
            assert_eq!(request.queue_number, format!("F{}", i + 1));
        }
    }
} 