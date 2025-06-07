use std::sync::atomic::{AtomicUsize, Ordering};
use crate::models::ChargingMode;

/// 排队号码生成器
pub struct QueueNumberGenerator {
    fast_counter: AtomicUsize,
    slow_counter: AtomicUsize,
}

impl QueueNumberGenerator {
    pub fn new() -> Self {
        Self {
            fast_counter: AtomicUsize::new(1),
            slow_counter: AtomicUsize::new(1),
        }
    }

    /// 生成新的排队号码
    pub fn generate(&self, mode: ChargingMode) -> String {
        match mode {
            ChargingMode::Fast => {
                let number = self.fast_counter.fetch_add(1, Ordering::SeqCst);
                format!("F{}", number)
            }
            ChargingMode::Slow => {
                let number = self.slow_counter.fetch_add(1, Ordering::SeqCst);
                format!("T{}", number)
            }
        }
    }

    /// 重置计数器（用于测试或系统重启）
    pub fn reset(&self) {
        self.fast_counter.store(1, Ordering::SeqCst);
        self.slow_counter.store(1, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_generation() {
        let generator = QueueNumberGenerator::new();
        
        // 测试快充号码生成
        assert_eq!(generator.generate(ChargingMode::Fast), "F1");
        assert_eq!(generator.generate(ChargingMode::Fast), "F2");
        
        // 测试慢充号码生成
        assert_eq!(generator.generate(ChargingMode::Slow), "T1");
        assert_eq!(generator.generate(ChargingMode::Slow), "T2");
    }

    #[test]
    fn test_reset() {
        let generator = QueueNumberGenerator::new();
        
        // 生成一些号码
        generator.generate(ChargingMode::Fast);
        generator.generate(ChargingMode::Slow);
        
        // 重置
        generator.reset();
        
        // 验证重置后的号码
        assert_eq!(generator.generate(ChargingMode::Fast), "F1");
        assert_eq!(generator.generate(ChargingMode::Slow), "T1");
    }
} 