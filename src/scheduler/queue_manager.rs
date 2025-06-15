use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::billing::FeeCalculator;
use crate::models::{
    ChargingMode, ChargingPile, ChargingRecord, ChargingRequest, PileStatus as ModelsPileStatus,
    RequestStatus, FAST_CHARGING_POWER, PILE_QUEUE_CAPACITY, SLOW_CHARGING_POWER,
    WAITING_AREA_CAPACITY,
};

/// 时间系统 - 30倍加速
#[derive(Debug)]
pub struct TimeSystem {
    real_start_time: Instant,
    system_start_time: DateTime<Utc>,
    acceleration_factor: f64,
}

impl TimeSystem {
    pub fn new() -> Self {
        Self {
            real_start_time: Instant::now(),
            system_start_time: Utc::now(),
            acceleration_factor: 30.0, // 30倍时间加速
        }
    }

    /// 获取当前系统时间（加速后的时间）
    pub fn current_time(&self) -> DateTime<Utc> {
        let real_elapsed = self.real_start_time.elapsed();
        let real_elapsed_seconds = real_elapsed.as_secs_f64();
        let system_elapsed_seconds = real_elapsed_seconds * self.acceleration_factor;

        self.system_start_time + Duration::seconds(system_elapsed_seconds as i64)
    }

    /// 计算两个时间点之间的小时数（系统时间）
    pub fn get_elapsed_hours(&self, start_time: DateTime<Utc>) -> f64 {
        let now = self.current_time();
        let elapsed = now.signed_duration_since(start_time);
        elapsed.num_seconds() as f64 / 3600.0
    }
}

/// 充电桩状态信息
#[derive(Debug, Clone)]
pub struct PileInfo {
    pub pile: Arc<RwLock<ChargingPile>>,
    pub queue: VecDeque<Arc<ChargingRequest>>,
    pub current_charging: Option<Arc<ChargingRequest>>,
    pub charging_start_time: Option<DateTime<Utc>>,
}

impl PileInfo {
    pub fn new(pile: Arc<RwLock<ChargingPile>>) -> Self {
        Self {
            pile,
            queue: VecDeque::new(),
            current_charging: None,
            charging_start_time: None,
        }
    }

    /// 获取充电功率
    pub async fn get_charging_power(&self) -> f64 {
        let pile = self.pile.read().await;
        match pile.mode {
            ChargingMode::Fast => FAST_CHARGING_POWER,
            ChargingMode::Slow => SLOW_CHARGING_POWER,
        }
    }

    /// 计算完成时间
    pub async fn calculate_completion_time(
        &self,
        new_request: &ChargingRequest,
        time_system: &TimeSystem,
    ) -> f64 {
        let power = self.get_charging_power().await;
        let remaining_amount = if let Some(ref current) = self.current_charging {
            let elapsed_hours = time_system.get_elapsed_hours(self.charging_start_time.unwrap());
            let remaining = current.amount - (elapsed_hours * power);
            remaining.max(0.0)
        } else {
            0.0
        };

        let queue_amount: f64 = self.queue.iter().map(|r| r.amount).sum();

        (remaining_amount + queue_amount + new_request.amount) / power
    }

    /// 检查是否有空间
    pub fn has_space(&self) -> bool {
        self.queue.len() < PILE_QUEUE_CAPACITY
    }

    /// 检查是否空闲
    pub fn is_idle(&self) -> bool {
        self.current_charging.is_none() && self.queue.is_empty()
    }

    /// 检查充电完成
    pub async fn check_charging_completion(
        &mut self,
        time_system: &TimeSystem,
    ) -> Option<(Arc<ChargingRequest>, DateTime<Utc>)> {
        if let (Some(ref charging), Some(start_time)) =
            (&self.current_charging, self.charging_start_time)
        {
            let elapsed_hours = time_system.get_elapsed_hours(start_time);
            let power = self.get_charging_power().await;
            let required_hours = charging.amount / power;

            if elapsed_hours >= required_hours {
                // 充电完成，保存开始时间然后清除状态
                let charging_start_time = start_time;
                let completed = self.current_charging.take().unwrap();
                self.charging_start_time = None;

                // 克隆并更新状态
                let mut completed_request = (*completed).clone();
                if let Err(e) = completed_request.complete_charging() {
                    println!("⚠️ 更新充电完成状态失败: {}", e);
                } else {
                    println!("✅ 请求状态已更新为已完成: {}", completed_request.user_id);
                }

                println!(
                    "🎉 车辆 {} 在充电桩 {} 完成充电! (充电量: {}度)",
                    completed_request.user_id,
                    self.pile.read().await.number,
                    completed_request.amount
                );

                let completed_arc = Arc::new(completed_request);

                // 立即开始下一辆车充电
                self.start_next_charging(time_system.current_time()).await;

                return Some((completed_arc, charging_start_time));
            }
        }
        None
    }

    /// 开始为下一辆车充电
    pub async fn start_next_charging(
        &mut self,
        current_time: DateTime<Utc>,
    ) -> Option<Arc<ChargingRequest>> {
        if self.current_charging.is_none() && !self.queue.is_empty() {
            let next_request = self.queue.pop_front().unwrap();

            // 克隆请求并更新状态为"充电中"
            let mut charging_request = (*next_request).clone();
            if let Err(e) = charging_request.start_charging() {
                println!("⚠️ 更新充电状态失败: {}", e);
            } else {
                println!("✅ 请求状态已更新为充电中: {}", charging_request.user_id);
            }

            let charging_request_arc = Arc::new(charging_request);
            self.current_charging = Some(charging_request_arc.clone());
            self.charging_start_time = Some(current_time);

            println!(
                "🔌 车辆 {} 在充电桩 {} 开始充电 (充电量: {}度)",
                charging_request_arc.user_id,
                self.pile.read().await.number,
                charging_request_arc.amount
            );

            return Some(charging_request_arc);
        }
        None
    }

    /// 获取充电进度
    pub async fn get_charging_progress(&self, time_system: &TimeSystem) -> Option<f64> {
        if let (Some(ref charging), Some(start_time)) =
            (&self.current_charging, self.charging_start_time)
        {
            let elapsed_hours = time_system.get_elapsed_hours(start_time);
            let total_hours = charging.amount / self.get_charging_power().await;
            Some((elapsed_hours / total_hours * 100.0).min(100.0))
        } else {
            None
        }
    }
}

/// 等候区队列管理器
pub struct QueueManager {
    // 等候区队列
    pub waiting_queue: RwLock<VecDeque<Arc<ChargingRequest>>>,

    // 充电桩信息，key为充电桩编号
    pub pile_infos: RwLock<HashMap<String, PileInfo>>,

    // 时间系统
    pub time_system: TimeSystem,

    // 数据库连接池
    pub db_pool: RwLock<Option<Arc<sqlx::MySqlPool>>>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            waiting_queue: RwLock::new(VecDeque::new()),
            pile_infos: RwLock::new(HashMap::new()),
            time_system: TimeSystem::new(),
            db_pool: RwLock::new(None),
        }
    }

    /// 设置数据库连接池
    pub async fn set_db_pool(&self, pool: Arc<sqlx::MySqlPool>) {
        let mut db_pool = self.db_pool.write().await;
        *db_pool = Some(pool);
        println!("✅ 队列管理器数据库连接池已设置");
    }

    /// 初始化充电桩
    pub async fn initialize_piles(&self) {
        let mut pile_infos = self.pile_infos.write().await;

        // 创建快充桩
        for i in 1..=2 {
            let pile = Arc::new(RwLock::new(ChargingPile::new(
                format!("F{}", i),
                ChargingMode::Fast,
            )));
            let number = pile.read().await.number.clone();
            pile_infos.insert(number, PileInfo::new(pile));
        }

        // 创建慢充桩
        for i in 1..=3 {
            let pile = Arc::new(RwLock::new(ChargingPile::new(
                format!("T{}", i),
                ChargingMode::Slow,
            )));
            let number = pile.read().await.number.clone();
            pile_infos.insert(number, PileInfo::new(pile));
        }

        println!("充电桩初始化完成: 2个快充桩 + 3个慢充桩");
    }

    // 添加充电桩
    pub async fn add_pile(&self, pile: Arc<RwLock<ChargingPile>>) {
        let mut pile_infos = self.pile_infos.write().await;
        let number = pile.read().await.number.clone();
        pile_infos.insert(number, PileInfo::new(pile));
    }

    // 添加充电请求到等候区
    pub async fn add_to_waiting_queue(&self, request: Arc<ChargingRequest>) -> Result<(), String> {
        let mut queue = self.waiting_queue.write().await;

        if queue.len() >= WAITING_AREA_CAPACITY {
            return Err("等候区已满".to_string());
        }

        queue.push_back(request.clone());
        println!(
            "车辆 {} 加入等候区，当前等待: {}",
            request.user_id,
            queue.len()
        );
        Ok(())
    }

    /// 系统tick - 检查充电完成并启动下一辆车
    pub async fn tick(&self) {
        let current_time = self.time_system.current_time();
        let mut pile_infos = self.pile_infos.write().await;

        for pile_info in pile_infos.values_mut() {
            // 检查充电完成
            if let Some((completed, start_time)) =
                pile_info.check_charging_completion(&self.time_system).await
            {
                println!("🎯 检测到充电完成，开始生成详单...");
                // 生成充电详单
                let end_time = current_time;
                let charging_time = self.time_system.get_elapsed_hours(start_time);

                // 计算费用
                let pile_number = pile_info.pile.read().await.number.clone();
                let billing_record = FeeCalculator::calculate_fee(
                    completed.user_id,
                    pile_number.clone(),
                    completed.amount,
                    start_time,
                    end_time,
                );

                // 创建充电详单
                let charging_record = ChargingRecord::new(
                    completed.user_id,
                    pile_number.clone(),
                    completed.mode.parse().unwrap_or(ChargingMode::Slow),
                    completed.amount,
                    charging_time,
                    billing_record.electricity_fee,
                    billing_record.service_fee,
                    start_time,
                    end_time,
                );

                // 保存充电详单到数据库
                println!(
                    "🔍 准备保存充电详单: 用户 {}, 充电桩 {}",
                    completed.user_id, pile_number
                );
                if let Some(pool) = self.db_pool.read().await.as_ref() {
                    println!("✅ 数据库连接池可用，开始保存充电详单");
                    if let Err(e) = charging_record.insert(pool).await {
                        println!("⚠️ 保存充电详单到数据库失败: {}", e);
                    }
                } else {
                    println!("⚠️ 数据库连接池未设置，无法保存充电详单");
                }

                // 更新充电桩统计信息
                let mut pile = pile_info.pile.write().await;
                pile.total_charge_count += 1;
                pile.total_charge_time += charging_time;
                pile.total_charge_amount += completed.amount;
                pile.total_charging_fee += billing_record.electricity_fee;
                pile.total_service_fee += billing_record.service_fee;

                // 保存统计信息回数据库
                if let Some(pool_arc) = self.db_pool.read().await.as_ref() {
                    let pool: &sqlx::MySqlPool = &**pool_arc; // 解引用 Arc -> Pool -> &Pool

                    let query = r#"
                        UPDATE charging_piles
                        SET 
                            status = 'Available',
                            total_charge_count = ?,
                            total_charge_time = ?,
                            total_charge_amount = ?,
                            total_charging_fee = ?,
                            total_service_fee = ?
                        WHERE number = ?
                    "#;

                    if let Err(e) = sqlx::query(query)
                        .bind(pile.total_charge_count)
                        .bind(pile.total_charge_time)
                        .bind(pile.total_charge_amount)
                        .bind(pile.total_charging_fee)
                        .bind(pile.total_service_fee)
                        .bind(&pile.number)
                        .execute(pool)
                        .await
                    {
                        println!("⚠️ 无法更新充电桩统计信息: {}", e);
                    } else {
                        println!("📦 成功更新充电桩 {} 的统计信息", &pile.number);
                    }
                } else {
                    println!("⚠️ 数据库连接池未设置，无法更新充电桩信息");
                }
            } else {
                // 只有在没有充电完成的情况下，才尝试启动下一辆车
                pile_info.start_next_charging(current_time).await;
            }
        }
    }

    /// 获取系统状态（供前端使用）
    pub async fn get_system_status(&self) -> SystemStatus {
        let waiting_queue = self.waiting_queue.read().await;
        let pile_infos = self.pile_infos.read().await;

        let mut pile_statuses = Vec::new();
        for (_, info) in pile_infos.iter() {
            let pile = info.pile.read().await;

            // 构建当前充电请求信息
            let current_request = info.current_charging.as_ref().map(|r| ChargingRequestInfo {
                id: r.id,
                user_id: r.user_id,
                mode: r.mode.clone(),
                amount: r.amount,
                queue_number: r.queue_number.clone(),
                status: r.status.clone(),
                created_at: r.created_at,
            });

            // 构建队列请求信息
            let queue_requests: Vec<ChargingRequestInfo> = info
                .queue
                .iter()
                .map(|r| ChargingRequestInfo {
                    id: r.id,
                    user_id: r.user_id,
                    mode: r.mode.clone(),
                    amount: r.amount,
                    queue_number: r.queue_number.clone(),
                    status: r.status.clone(),
                    created_at: r.created_at,
                })
                .collect();

            pile_statuses.push(PileStatusInfo {
                pile_number: pile.number.clone(),
                pile_mode: pile.mode,
                current_charging_user: info.current_charging.as_ref().map(|r| r.user_id),
                queue_users: info.queue.iter().map(|r| r.user_id).collect(),
                queue_count: info.queue.len(),
                is_idle: info.is_idle(),
                charging_progress: info.get_charging_progress(&self.time_system).await,
                current_request,
                queue_requests,
            });
        }

        SystemStatus {
            current_time: self.time_system.current_time(),
            fast_waiting_count: waiting_queue.iter().filter(|r| r.mode == "Fast").count(),
            slow_waiting_count: waiting_queue.iter().filter(|r| r.mode == "Slow").count(),
            fast_waiting_requests: waiting_queue
                .iter()
                .filter(|r| r.mode == "Fast")
                .map(|r| r.user_id)
                .collect(),
            slow_waiting_requests: waiting_queue
                .iter()
                .filter(|r| r.mode == "Slow")
                .map(|r| r.user_id)
                .collect(),
            pile_statuses,
        }
    }

    /// 获取系统状态（供前端使用）
    pub async fn get_status(&self) -> SystemRealTimeStatusForQueue {
        let pile_infos = self.pile_infos.read().await;
        let waiting_queue = self.waiting_queue.read().await;

        let mut pile_statuses = Vec::new();
        for info in pile_infos.values() {
            let pile = info.pile.read().await;
            pile_statuses.push(PileRealTimeStatus {
                pile_number: pile.number.clone(),
                pile_mode: pile.mode,
                is_idle: info.current_charging.is_none() && info.queue.is_empty(),
                current_charging_user: info.current_charging.as_ref().map(|req| req.user_id),
                current_request: info.current_charging.as_ref().map(|req| (**req).clone()),
                queue_count: info.queue.len(),
                queue_requests: info.queue.iter().map(|req| (**req).clone()).collect(),
                charging_progress: info.get_charging_progress(&self.time_system).await,
            });
        }

        let fast_waiting_requests: Vec<Arc<ChargingRequest>> = waiting_queue
            .iter()
            .filter(|r| r.mode == "Fast")
            .cloned()
            .collect();
        let slow_waiting_requests: Vec<Arc<ChargingRequest>> = waiting_queue
            .iter()
            .filter(|r| r.mode == "Slow")
            .cloned()
            .collect();

        SystemRealTimeStatusForQueue {
            pile_statuses,
            fast_waiting_count: fast_waiting_requests.len(),
            slow_waiting_count: slow_waiting_requests.len(),
            fast_waiting_requests,
            slow_waiting_requests,
        }
    }
}

/// 系统状态（供前端使用）
#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub current_time: DateTime<Utc>,
    pub fast_waiting_count: usize,
    pub slow_waiting_count: usize,
    pub fast_waiting_requests: Vec<Uuid>,
    pub slow_waiting_requests: Vec<Uuid>,
    pub pile_statuses: Vec<PileStatusInfo>,
}

/// 充电桩状态（供前端使用）
#[derive(Debug, Serialize)]
pub struct PileStatusInfo {
    pub pile_number: String,
    pub pile_mode: ChargingMode,
    pub current_charging_user: Option<Uuid>,
    pub queue_users: Vec<Uuid>,
    pub queue_count: usize,
    pub is_idle: bool,
    pub charging_progress: Option<f64>, // 充电进度百分比
    pub current_request: Option<ChargingRequestInfo>, // 当前充电请求的完整信息
    pub queue_requests: Vec<ChargingRequestInfo>, // 队列中的请求信息
}

/// 充电请求信息（供前端使用）
#[derive(Debug, Serialize)]
pub struct ChargingRequestInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub mode: String,
    pub amount: f64, // 用户请求的充电量
    pub queue_number: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SystemRealTimeStatusForQueue {
    pub pile_statuses: Vec<PileRealTimeStatus>,
    pub fast_waiting_count: usize,
    pub slow_waiting_count: usize,
    pub fast_waiting_requests: Vec<Arc<ChargingRequest>>,
    pub slow_waiting_requests: Vec<Arc<ChargingRequest>>,
}

#[derive(Debug, Clone)]
pub struct PileRealTimeStatus {
    pub pile_number: String,
    pub pile_mode: ChargingMode,
    pub is_idle: bool,
    pub current_charging_user: Option<Uuid>,
    pub current_request: Option<ChargingRequest>,
    pub queue_count: usize,
    pub queue_requests: Vec<ChargingRequest>,
    pub charging_progress: Option<f64>,
}
