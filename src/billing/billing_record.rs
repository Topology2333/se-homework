use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingRecord {
    pub user_id: Uuid,
    pub pile_id: String,
    pub charge_amount: f64,     // 充电量（度）
    pub charge_time: f64,       // 充电时长（小时）
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub electricity_fee: f64,   // 电费
    pub service_fee: f64,       // 服务费
    pub total_fee: f64,         // 总费用
}

impl BillingRecord {
    pub fn new(
        user_id: Uuid,
        pile_id: String,
        charge_amount: f64,
        charge_time: f64,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        electricity_fee: f64,
        service_fee: f64,
    ) -> Self {
        let total_fee = electricity_fee + service_fee;
        
        Self {
            user_id,
            pile_id,
            charge_amount,
            charge_time,
            start_time,
            end_time,
            electricity_fee,
            service_fee,
            total_fee,
        }
    }
} 