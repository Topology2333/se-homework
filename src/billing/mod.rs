mod fee_calculator;
mod billing_record;
mod time_slot;

pub use fee_calculator::FeeCalculator;
pub use billing_record::BillingRecord;
pub use time_slot::TimeSlot;

// 电费费率（元/度）
pub const PEAK_RATE: f64 = 1.0;    // 峰时
pub const FLAT_RATE: f64 = 0.7;    // 平时
pub const VALLEY_RATE: f64 = 0.4;  // 谷时
pub const SERVICE_RATE: f64 = 0.8;  // 服务费 