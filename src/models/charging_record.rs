use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::ChargingMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargingRecord {
    pub id: Uuid,                    // 详单ID
    pub user_id: Uuid,              // 用户ID
    pub pile_id: String,            // 充电桩编号
    pub mode: ChargingMode,         // 充电模式
    pub charging_amount: f64,       // 充电量（度）
    pub charging_time: f64,         // 充电时长（小时）
    pub charging_fee: f64,          // 充电费用
    pub service_fee: f64,           // 服务费用
    pub total_fee: f64,             // 总费用
    pub start_time: chrono::DateTime<chrono::Utc>,  // 开始时间
    pub end_time: chrono::DateTime<chrono::Utc>,    // 结束时间
    pub created_at: chrono::DateTime<chrono::Utc>,  // 详单生成时间
}

impl ChargingRecord {
    pub fn new(
        user_id: Uuid,
        pile_id: String,
        mode: ChargingMode,
        charging_amount: f64,
        charging_time: f64,
        charging_fee: f64,
        service_fee: f64,
        start_time: chrono::DateTime<chrono::Utc>,
        end_time: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            pile_id,
            mode,
            charging_amount,
            charging_time,
            charging_fee,
            service_fee,
            total_fee: charging_fee + service_fee,
            start_time,
            end_time,
            created_at: chrono::Utc::now(),
        }
    }
} 