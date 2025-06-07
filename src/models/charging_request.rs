use crate::models::{ChargingMode, RequestStatus};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargingRequest {
    pub id: Uuid,                    // 请求ID
    pub user_id: Uuid,              // 用户ID
    pub mode: ChargingMode,         // 充电模式
    pub amount: f64,                // 请求充电量（度）
    pub queue_number: String,       // 排队号码（F1、F2、T1、T2等）
    pub status: RequestStatus,      // 请求状态
    pub created_at: DateTime<Utc>,  // 创建时间
}

impl ChargingRequest {
    pub fn new(user_id: Uuid, mode: ChargingMode, amount: f64, queue_number: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            mode,
            amount,
            queue_number,
            status: RequestStatus::Waiting,
            created_at: Utc::now(),
        }
    }

    /// 开始充电
    pub fn start_charging(&mut self) -> Result<(), String> {
        match self.status {
            RequestStatus::Waiting => {
                self.status = RequestStatus::Charging;
                Ok(())
            }
            _ => Err("请求状态不正确".to_string()),
        }
    }

    /// 完成充电
    pub fn complete_charging(&mut self) -> Result<(), String> {
        match self.status {
            RequestStatus::Charging => {
                self.status = RequestStatus::Completed;
                Ok(())
            }
            _ => Err("请求状态不正确".to_string()),
        }
    }

    /// 取消请求
    pub fn cancel(&mut self) -> Result<(), String> {
        match self.status {
            RequestStatus::Waiting | RequestStatus::Charging => {
                self.status = RequestStatus::Cancelled;
                Ok(())
            }
            _ => Err("请求状态不正确".to_string()),
        }
    }

    pub fn update_amount(&mut self, new_amount: f64) {
        self.amount = new_amount;
    }

    pub fn update_mode(&mut self, new_mode: ChargingMode, new_queue_number: String) {
        self.mode = new_mode;
        self.queue_number = new_queue_number;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_request() {
        let user_id = Uuid::new_v4();
        let request = ChargingRequest::new(
            user_id,
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        );

        assert_eq!(request.user_id, user_id);
        assert_eq!(request.mode, ChargingMode::Fast);
        assert_eq!(request.amount, 30.0);
        assert_eq!(request.queue_number, "F1");
        assert_eq!(request.status, RequestStatus::Waiting);
    }

    #[test]
    fn test_request_lifecycle() {
        let mut request = ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        );

        // 开始充电
        request.start_charging().unwrap();
        assert_eq!(request.status, RequestStatus::Charging);

        // 完成充电
        request.complete_charging().unwrap();
        assert_eq!(request.status, RequestStatus::Completed);
    }

    #[test]
    fn test_cancel_request() {
        let mut request = ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        );

        // 等待状态下取消
        request.cancel().unwrap();
        assert_eq!(request.status, RequestStatus::Cancelled);

        // 已取消状态下不能再取消
        assert!(request.cancel().is_err());
    }
} 