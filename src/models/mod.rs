mod charging_pile;
mod vehicle;
mod charging_request;
mod charging_record;
mod user;

pub use charging_pile::*;
pub use vehicle::*;
pub use charging_request::*;
pub use charging_record::*;
pub use user::*;

// 充电模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChargingMode {
    Fast,  // 快充
    Slow,  // 慢充
}

// 充电桩状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PileStatus {
    Working,    // 正常工作
    Fault,      // 故障
    Shutdown,   // 关闭
}

// 时段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeSlotType {
    Peak,       // 峰时 10:00-15:00, 18:00-21:00
    Normal,     // 平时 7:00-10:00, 15:00-18:00, 21:00-23:00
    Valley,     // 谷时 23:00-次日7:00
}

// 系统常量
pub const WAITING_AREA_CAPACITY: usize = 6;        // 等候区容量
pub const PILE_QUEUE_CAPACITY: usize = 2;          // 每个充电桩队列容量
pub const FAST_CHARGING_POWER: f64 = 30.0;         // 快充功率（度/小时）
pub const SLOW_CHARGING_POWER: f64 = 7.0;          // 慢充功率（度/小时）
pub const SERVICE_FEE_RATE: f64 = 0.8;            // 服务费率（元/度）

// 电价常量
pub const PEAK_PRICE: f64 = 1.0;      // 峰时电价
pub const NORMAL_PRICE: f64 = 0.7;    // 平时电价
pub const VALLEY_PRICE: f64 = 0.4;    // 谷时电价

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_charging_pile() {
        let mut pile = ChargingPile::new("A".to_string(), ChargingMode::Fast);
        assert_eq!(pile.status, PileStatus::Shutdown);
        assert_eq!(pile.get_power(), FAST_CHARGING_POWER);

        pile.start();
        assert_eq!(pile.status, PileStatus::Working);
        
        pile.update_statistics(1.0, 30.0, 30.0, 24.0);
        assert_eq!(pile.total_charging_times, 1);
        assert_eq!(pile.total_charging_amount, 30.0);
    }

    #[test]
    fn test_charging_request() {
        let user_id = Uuid::new_v4();
        let mut request = ChargingRequest::new(
            user_id,
            ChargingMode::Fast,
            30.0,
            "F1".to_string(),
        );

        assert_eq!(request.status, RequestStatus::Waiting);
        request.start_charging();
        assert_eq!(request.status, RequestStatus::Charging);
        request.complete();
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