use chrono::{DateTime, Timelike, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSlot {
    Peak,    // 峰时：10:00-15:00，18:00-21:00
    Flat,    // 平时：7:00-10:00，15:00-18:00，21:00-23:00
    Valley,  // 谷时：23:00-次日7:00
}

impl TimeSlot {
    /// 判断给定时间属于哪个时段
    pub fn from_time(time: &DateTime<Utc>) -> Self {
        let hour = time.hour();
        
        match hour {
            // 峰时段
            10..=14 | 18..=20 => TimeSlot::Peak,
            
            // 平时段
            7..=9 | 15..=17 | 21..=22 => TimeSlot::Flat,
            
            // 谷时段
            23 | 0..=6 => TimeSlot::Valley,
            
            // 不应该出现的情况
            _ => unreachable!("Invalid hour: {}", hour),
        }
    }

    /// 获取当前时段的费率
    pub fn get_rate(&self) -> f64 {
        use super::{PEAK_RATE, FLAT_RATE, VALLEY_RATE};
        
        match self {
            TimeSlot::Peak => PEAK_RATE,
            TimeSlot::Flat => FLAT_RATE,
            TimeSlot::Valley => VALLEY_RATE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use crate::billing::{PEAK_RATE, FLAT_RATE, VALLEY_RATE};

    #[test]
    fn test_time_slot_classification() {
        // 测试峰时段
        let peak_time1 = Utc.with_ymd_and_hms(2024, 3, 1, 11, 0, 0).unwrap();
        assert_eq!(TimeSlot::from_time(&peak_time1), TimeSlot::Peak);
        
        let peak_time2 = Utc.with_ymd_and_hms(2024, 3, 1, 19, 30, 0).unwrap();
        assert_eq!(TimeSlot::from_time(&peak_time2), TimeSlot::Peak);

        // 测试平时段
        let flat_time1 = Utc.with_ymd_and_hms(2024, 3, 1, 8, 0, 0).unwrap();
        assert_eq!(TimeSlot::from_time(&flat_time1), TimeSlot::Flat);
        
        let flat_time2 = Utc.with_ymd_and_hms(2024, 3, 1, 16, 30, 0).unwrap();
        assert_eq!(TimeSlot::from_time(&flat_time2), TimeSlot::Flat);

        // 测试谷时段
        let valley_time1 = Utc.with_ymd_and_hms(2024, 3, 1, 23, 30, 0).unwrap();
        assert_eq!(TimeSlot::from_time(&valley_time1), TimeSlot::Valley);
        
        let valley_time2 = Utc.with_ymd_and_hms(2024, 3, 1, 5, 0, 0).unwrap();
        assert_eq!(TimeSlot::from_time(&valley_time2), TimeSlot::Valley);
    }

    #[test]
    fn test_rate_mapping() {
        assert_eq!(TimeSlot::Peak.get_rate(), PEAK_RATE);
        assert_eq!(TimeSlot::Flat.get_rate(), FLAT_RATE);
        assert_eq!(TimeSlot::Valley.get_rate(), VALLEY_RATE);
    }
} 