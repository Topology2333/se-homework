use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::ChargingMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargingRequest {
    pub id: Uuid,                    // 请求ID
    pub user_id: Uuid,              // 用户ID
    pub mode: ChargingMode,         // 充电模式
    pub amount: f64,                // 请求充电量（度）
    pub queue_number: String,       // 排队号码（F1、F2、T1、T2等）
    pub created_at: chrono::DateTime<chrono::Utc>,  // 创建时间
    pub status: RequestStatus,      // 请求状态
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestStatus {
    Waiting,        // 等待中
    Charging,       // 充电中
    Completed,      // 已完成
    Cancelled,      // 已取消
}

impl ChargingRequest {
    pub fn new(user_id: Uuid, mode: ChargingMode, amount: f64, queue_number: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            mode,
            amount,
            queue_number,
            created_at: chrono::Utc::now(),
            status: RequestStatus::Waiting,
        }
    }

    pub fn start_charging(&mut self) {
        self.status = RequestStatus::Charging;
    }

    pub fn complete(&mut self) {
        self.status = RequestStatus::Completed;
    }

    pub fn cancel(&mut self) {
        self.status = RequestStatus::Cancelled;
    }

    pub fn update_amount(&mut self, new_amount: f64) {
        self.amount = new_amount;
    }

    pub fn update_mode(&mut self, new_mode: ChargingMode, new_queue_number: String) {
        self.mode = new_mode;
        self.queue_number = new_queue_number;
    }
} 