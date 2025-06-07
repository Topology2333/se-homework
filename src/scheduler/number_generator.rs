use crate::models::ChargingMode;
use parking_lot::RwLock;
use std::collections::HashMap;

/// 排队号码生成器
pub struct NumberGenerator {
    // 每种模式的当前序号
    counters: RwLock<HashMap<ChargingMode, u32>>,
}

impl NumberGenerator {
    pub fn new() -> Self {
        let mut counters = HashMap::new();
        counters.insert(ChargingMode::Fast, 0);
        counters.insert(ChargingMode::Slow, 0);
        
        Self {
            counters: RwLock::new(counters),
        }
    }

    /// 生成新的排队号码
    pub fn generate_number(&self, mode: ChargingMode) -> Result<String, String> {
        let mut counters = self.counters.write();
        
        let current = counters.get_mut(&mode).ok_or("Invalid charging mode")?;
        *current += 1;
        
        let prefix = match mode {
            ChargingMode::Fast => "F",
            ChargingMode::Slow => "T",
        };
        
        Ok(format!("{}{}", prefix, current))
    }

    /// 重置指定模式的计数器
    pub fn reset_counter(&self, mode: ChargingMode) {
        let mut counters = self.counters.write();
        if let Some(counter) = counters.get_mut(&mode) {
            *counter = 0;
        }
    }

    /// 重置所有计数器
    pub fn reset_all(&self) {
        let mut counters = self.counters.write();
        for counter in counters.values_mut() {
            *counter = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_number() {
        let generator = NumberGenerator::new();
        
        // 测试快充号码生成
        assert_eq!(generator.generate_number(ChargingMode::Fast).unwrap(), "F1");
        assert_eq!(generator.generate_number(ChargingMode::Fast).unwrap(), "F2");
        
        // 测试慢充号码生成
        assert_eq!(generator.generate_number(ChargingMode::Slow).unwrap(), "T1");
        assert_eq!(generator.generate_number(ChargingMode::Slow).unwrap(), "T2");
    }

    #[test]
    fn test_reset_counter() {
        let generator = NumberGenerator::new();
        
        // 生成一些号码
        generator.generate_number(ChargingMode::Fast).unwrap();
        generator.generate_number(ChargingMode::Fast).unwrap();
        
        // 重置快充计数器
        generator.reset_counter(ChargingMode::Fast);
        
        // 验证重置后的号码
        assert_eq!(generator.generate_number(ChargingMode::Fast).unwrap(), "F1");
    }

    #[test]
    fn test_concurrent_mode_generation() {
        let generator = NumberGenerator::new();
        
        // 交替生成快充和慢充号码
        assert_eq!(generator.generate_number(ChargingMode::Fast).unwrap(), "F1");
        assert_eq!(generator.generate_number(ChargingMode::Slow).unwrap(), "T1");
        assert_eq!(generator.generate_number(ChargingMode::Fast).unwrap(), "F2");
        assert_eq!(generator.generate_number(ChargingMode::Slow).unwrap(), "T2");
    }

    #[test]
    fn test_reset_all_counters() {
        let generator = NumberGenerator::new();
        
        // 生成一些号码
        generator.generate_number(ChargingMode::Fast).unwrap();
        generator.generate_number(ChargingMode::Fast).unwrap();
        generator.generate_number(ChargingMode::Slow).unwrap();
        
        // 重置所有计数器
        generator.reset_all();
        
        // 验证所有计数器都被重置
        assert_eq!(generator.generate_number(ChargingMode::Fast).unwrap(), "F1");
        assert_eq!(generator.generate_number(ChargingMode::Slow).unwrap(), "T1");
    }

    #[test]
    fn test_large_number_generation() {
        let generator = NumberGenerator::new();
        
        // 生成大量号码
        for i in 1..=1000 {
            let number = generator.generate_number(ChargingMode::Fast).unwrap();
            assert_eq!(number, format!("F{}", i));
        }
    }

    #[test]
    fn test_selective_reset() {
        let generator = NumberGenerator::new();
        
        // 生成一些号码
        generator.generate_number(ChargingMode::Fast).unwrap(); // F1
        generator.generate_number(ChargingMode::Slow).unwrap(); // T1
        generator.generate_number(ChargingMode::Fast).unwrap(); // F2
        
        // 只重置快充计数器
        generator.reset_counter(ChargingMode::Fast);
        
        // 验证只有快充被重置
        assert_eq!(generator.generate_number(ChargingMode::Fast).unwrap(), "F1");
        assert_eq!(generator.generate_number(ChargingMode::Slow).unwrap(), "T2");
    }
} 