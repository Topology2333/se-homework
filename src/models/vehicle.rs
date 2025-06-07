use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: Uuid,                // 车辆ID
    pub user_id: Uuid,          // 所属用户ID
    pub battery_capacity: f64,   // 电池总容量（度）
    pub current_battery: f64,    // 当前电量（度）
    pub created_at: chrono::DateTime<chrono::Utc>,  // 创建时间
}

impl Vehicle {
    pub fn new(user_id: Uuid, battery_capacity: f64, current_battery: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            battery_capacity,
            current_battery,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn update_battery(&mut self, new_battery: f64) {
        self.current_battery = new_battery.min(self.battery_capacity);
    }

    pub fn can_charge(&self, request_amount: f64) -> bool {
        self.current_battery + request_amount <= self.battery_capacity
    }
} 