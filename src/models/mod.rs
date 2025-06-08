mod charging_pile;
mod vehicle;
mod charging_request;
mod charging_record;
pub mod user;

pub use charging_pile::*;
pub use vehicle::*;
pub use charging_request::*;
pub use charging_record::*;
pub use user::*;

use serde::{Serialize, Deserialize};
use sqlx::Type; // 引入 sqlx::Type trait

// 充电模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChargingMode {
    Fast,  // 快充
    Slow,  // 慢充
}

impl From<String> for ChargingMode {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Fast" => ChargingMode::Fast,
            "Slow" => ChargingMode::Slow,
            _ => panic!("Unknown charging mode: {}", s),
        }
    }
}

// 充电桩状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PileStatus {
    Available,       // 空闲
    Charging,        // 充电中
    Fault,           // 故障
    Shutdown,        // 关机
}

// 时段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSlotType {
    Peak,     // 峰时 10:00-15:00, 18:00-21:00
    Normal,   // 平时 7:00-10:00, 15:00-18:00, 21:00-23:00
    Valley,   // 谷时 23:00-次日7:00
}

// 系统常量
pub const WAITING_AREA_CAPACITY: usize = 6;         // 等候区容量
pub const FAST_CHARGING_PILES: usize = 2;           // 快充桩数量
pub const SLOW_CHARGING_PILES: usize = 4;           // 慢充桩数量
pub const PILE_QUEUE_CAPACITY: usize = 2;           // 每个充电桩队列容量
pub const FAST_CHARGING_POWER: f64 = 30.0;          // 快充功率（度/小时）
pub const SLOW_CHARGING_POWER: f64 = 7.0;           // 慢充功率（度/小时）
pub const SERVICE_FEE_RATE: f64 = 0.8;             // 服务费率（元/度）

// 电价常量
pub const PEAK_PRICE: f64 = 1.0;      // 峰时电价
pub const NORMAL_PRICE: f64 = 0.7;    // 平时电价
pub const VALLEY_PRICE: f64 = 0.4;    // 谷时电价

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestStatus {
    Waiting,         // 等待中
    Charging,        // 充电中
    Completed,       // 已完成
    Cancelled,       // 已取消
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;
    use crate::models::charging_pile::ChargingPile;
    use crate::models::charging_request::ChargingRequest;

    #[test]
    fn test_charging_pile() {
        let mut pile = ChargingPile::new("C1".to_string(), ChargingMode::Fast);
        
        // 测试初始状态
        assert_eq!(pile.number, "C1");
        assert_eq!(pile.mode, ChargingMode::Fast);
        assert_eq!(pile.status, PileStatus::Available);
        
        // 测试充电过程
        pile.start_charging().unwrap();
        assert_eq!(pile.status, PileStatus::Charging);
        
        pile.stop_charging(1.0, 30.0).unwrap();
        assert_eq!(pile.status, PileStatus::Available);
        assert_eq!(pile.total_charge_count, 1);
        assert_eq!(pile.total_charge_amount, 30.0);
    }

    #[test]
    fn test_charging_request() {
        let mut request = ChargingRequest::new(
            Uuid::new_v4(),
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        );
        
        // 测试初始状态
        assert_eq!(request.status, RequestStatus::Waiting);
        
        // 测试状态转换
        request.start_charging().unwrap();
        assert_eq!(request.status, RequestStatus::Charging);
        
        request.complete_charging().unwrap();
        assert_eq!(request.status, RequestStatus::Completed);
    }

    #[test]
    fn test_vehicle() {
        let user_id = Uuid::new_v4();
        let mut vehicle = Vehicle::new(user_id, 100.0, 20.0);
        
        assert!(vehicle.can_charge(80.0));
        assert!(!vehicle.can_charge(81.0));
        
        vehicle.update_battery(50.0);
        assert_eq!(vehicle.current_battery, 50.0);
    }
}