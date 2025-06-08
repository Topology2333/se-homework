use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use super::{
    TimeSlot,
    BillingRecord,
    SERVICE_RATE,
};

pub struct FeeCalculator;

impl FeeCalculator {
    /// 计算充电费用
    pub fn calculate_fee(
        user_id: Uuid,
        pile_id: String,
        charge_amount: f64,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> BillingRecord {
        // 计算充电时长（小时）
        let duration = end_time - start_time;
        let charge_time = duration.num_minutes() as f64 / 60.0;

        // 计算每个时段的电量占比
        let mut current_time = start_time;
        let mut electricity_fee = 0.0;
        let mut remaining_amount = charge_amount;
        let total_minutes = duration.num_minutes() as f64;
        
        while current_time < end_time && remaining_amount > 0.0 {
            let time_slot = TimeSlot::from_time(&current_time);
            let rate = time_slot.get_rate();
            
            // 计算当前时段的结束时间（每分钟计算一次）
            let next_minute = current_time + Duration::minutes(1);
            let period_end = if next_minute > end_time {
                end_time
            } else {
                next_minute
            };
            
            // 计算当前分钟的电量和费用
            let period_ratio = 1.0 / total_minutes;  // 每分钟的比例
            let period_amount = charge_amount * period_ratio;
            electricity_fee += period_amount * rate;
            
            remaining_amount -= period_amount;
            current_time = period_end;
        }

        // 计算服务费
        let service_fee = charge_amount * SERVICE_RATE;

        // 生成账单记录
        BillingRecord::new(
            user_id,
            pile_id,
            charge_amount,
            charge_time,
            start_time,
            end_time,
            electricity_fee,
            service_fee,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_fee_calculation() {
        let user_id = Uuid::new_v4();
        let pile_id = "A1".to_string();
        
        // 测试峰时段充电
        let start_time = Utc.with_ymd_and_hms(2024, 3, 1, 11, 0, 0).unwrap();  // 峰时段
        let end_time = Utc.with_ymd_and_hms(2024, 3, 1, 12, 0, 0).unwrap();    // 峰时段
        
        let record = FeeCalculator::calculate_fee(
            user_id,
            pile_id.clone(),
            30.0,  // 充电量30度
            start_time,
            end_time,
        );

        // 峰时电价1.0元/度，服务费0.8元/度
        assert_eq!(record.electricity_fee, 30.0);  // 30度 * 1.0元/度
        assert_eq!(record.service_fee, 24.0);      // 30度 * 0.8元/度
        assert_eq!(record.total_fee, 54.0);        // 30.0 + 24.0
        
        // 测试跨时段充电（峰时+平时）
        let start_time = Utc.with_ymd_and_hms(2024, 3, 1, 14, 30, 0).unwrap();  // 峰时段
        let end_time = Utc.with_ymd_and_hms(2024, 3, 1, 15, 30, 0).unwrap();    // 平时段
        
        let record = FeeCalculator::calculate_fee(
            user_id,
            pile_id,
            30.0,  // 充电量30度
            start_time,
            end_time,
        );

        // 半小时峰时(1.0元/度)，半小时平时(0.7元/度)
        let expected_fee = 30.0 * 0.5 * 1.0 + 30.0 * 0.5 * 0.7;  // 25.5元
        assert!((record.electricity_fee - expected_fee).abs() < 0.01);
        assert_eq!(record.service_fee, 24.0);  // 30度 * 0.8元/度
    }
} 