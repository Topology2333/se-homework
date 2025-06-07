use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{ChargingMode, PileStatus, FAST_CHARGING_POWER, SLOW_CHARGING_POWER};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChargingPile {
    pub id: Uuid,                    // 充电桩ID
    pub pile_id: String,             // 充电桩编号（A/B/C/D/E）
    pub mode: ChargingMode,          // 充电模式
    pub status: PileStatus,          // 当前状态
    pub total_charging_times: u32,   // 累计充电次数
    pub total_charging_time: f64,    // 累计充电时长（小时）
    pub total_charging_amount: f64,  // 累计充电量（度）
    pub total_charging_fee: f64,     // 累计充电费用
    pub total_service_fee: f64,      // 累计服务费用
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,  // 启动时间
}

impl ChargingPile {
    pub fn new(pile_id: String, mode: ChargingMode) -> Self {
        Self {
            id: Uuid::new_v4(),
            pile_id,
            mode,
            status: PileStatus::Shutdown,
            total_charging_times: 0,
            total_charging_time: 0.0,
            total_charging_amount: 0.0,
            total_charging_fee: 0.0,
            total_service_fee: 0.0,
            started_at: None,
        }
    }

    pub fn get_power(&self) -> f64 {
        match self.mode {
            ChargingMode::Fast => FAST_CHARGING_POWER,
            ChargingMode::Slow => SLOW_CHARGING_POWER,
        }
    }

    pub fn start(&mut self) {
        self.status = PileStatus::Working;
        self.started_at = Some(chrono::Utc::now());
    }

    pub fn shutdown(&mut self) {
        self.status = PileStatus::Shutdown;
    }

    pub fn set_fault(&mut self) {
        self.status = PileStatus::Fault;
    }

    pub fn update_statistics(&mut self, charging_time: f64, charging_amount: f64, charging_fee: f64, service_fee: f64) {
        self.total_charging_times += 1;
        self.total_charging_time += charging_time;
        self.total_charging_amount += charging_amount;
        self.total_charging_fee += charging_fee;
        self.total_service_fee += service_fee;
    }
} 