pub mod dispatcher;
mod number_generator;
pub mod queue_manager;

pub use dispatcher::Dispatcher;
pub use number_generator::QueueNumberGenerator;
pub use queue_manager::{QueueManager, PileStatusInfo};

use crate::models::{ChargingMode, ChargingPile, ChargingRequest};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 充电调度系统
pub struct ChargingScheduler {
    pub queue_manager: Arc<QueueManager>,
    number_generator: Arc<QueueNumberGenerator>,
    dispatcher: Arc<Dispatcher>,
    is_running: Arc<RwLock<bool>>,
    db_pool: Option<Arc<sqlx::MySqlPool>>,
}

impl ChargingScheduler {
    pub fn new() -> Self {
        let queue_manager = Arc::new(QueueManager::new());
        let number_generator = Arc::new(QueueNumberGenerator::new());
        let dispatcher = Arc::new(Dispatcher::new(queue_manager.clone()));

        Self {
            queue_manager,
            number_generator,
            dispatcher,
            is_running: Arc::new(RwLock::new(false)),
            db_pool: None,
        }
    }

    /// 设置数据库连接池
    pub fn with_db_pool(mut self, pool: Arc<sqlx::MySqlPool>) -> Self {
        self.db_pool = Some(pool);
        self
    }

    /// 启动调度系统
    pub async fn start(&self) -> Result<(), String> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            return Err("调度系统已经在运行".to_string());
        }

        // 设置数据库连接池到队列管理器
        if let Some(pool) = &self.db_pool {
            self.queue_manager.set_db_pool(pool.clone()).await;
            println!("✅ 数据库连接池已设置到队列管理器");
        } else {
            println!("⚠️ 调度器没有数据库连接池");
        }

        // 初始化充电桩
        self.queue_manager.initialize_piles().await;
        
        // 启动叫号服务
        self.dispatcher.start_calling().await;
        
        *is_running = true;
        
        // 启动后台tick循环
        let dispatcher = self.dispatcher.clone();
        let is_running_clone = self.is_running.clone();
        
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(100)); // 0.1秒tick一次
            
            loop {
                interval.tick().await;
                
                // 检查是否还在运行
                if !*is_running_clone.read().await {
                    break;
                }
                
                // 执行系统tick
                dispatcher.tick().await;
            }
            
            println!("调度系统后台任务已停止");
        });
        
        println!("🚀 充电调度系统已启动");
        Ok(())
    }

    /// 停止调度系统
    pub async fn stop(&self) -> Result<(), String> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            return Err("调度系统未运行".to_string());
        }

        // 停止叫号服务
        self.dispatcher.stop_calling().await;
        
        *is_running = false;
        
        println!("🛑 充电调度系统已停止");
        Ok(())
    }

    /// 提交充电请求
    pub async fn submit_request(&self, mut request: ChargingRequest) -> Result<(), String> {
        // 生成排队号码
        let queue_number = self.number_generator.generate(request.mode.parse()?);
        request.queue_number = queue_number;
        
        println!("生成排队号码: {} 用户: {}", request.queue_number, request.user_id);
        
        // 添加到等候区
        self.queue_manager.add_to_waiting_queue(Arc::new(request)).await?;
        
        Ok(())
    }

    /// 处理充电桩故障
    pub async fn handle_pile_fault(&self, pile_id: &str) -> Result<(), String> {
        self.dispatcher.handle_pile_fault(pile_id).await;
        Ok(())
    }

    /// 处理充电桩恢复
    pub async fn handle_pile_recovery(&self, pile_id: &str) -> Result<(), String> {
        self.dispatcher.handle_pile_recovery(pile_id).await;
        Ok(())
    }

    /// 获取系统状态（前端接口）
    pub async fn get_system_status(&self) -> SystemStatus {
        let queue_status = self.queue_manager.get_status().await;
        SystemStatus {
            pile_statuses: queue_status.pile_statuses.into_iter().map(|p| PileStatus {
                pile_number: p.pile_number,
                pile_mode: p.pile_mode,
                is_idle: p.is_idle,
                current_charging_user: p.current_charging_user,
                current_request: p.current_request,
                queue_count: p.queue_count,
                queue_requests: p.queue_requests,
                charging_progress: p.charging_progress,
            }).collect(),
            fast_waiting_count: queue_status.fast_waiting_count,
            slow_waiting_count: queue_status.slow_waiting_count,
            fast_waiting_requests: queue_status.fast_waiting_requests.iter().map(|r| (**r).clone()).collect(),
            slow_waiting_requests: queue_status.slow_waiting_requests.iter().map(|r| (**r).clone()).collect(),
        }
    }

    /// 获取调度器状态
    pub async fn get_scheduler_status(&self) -> SchedulerStatus {
        SchedulerStatus {
            is_running: *self.is_running.read().await,
            is_calling: self.dispatcher.is_calling().await,
        }
    }

    /// 手动触发系统tick（用于测试）
    pub async fn manual_tick(&self) {
        self.queue_manager.tick().await;
    }

    /// 取消充电请求
    pub async fn cancel_request(&self, request_id: Uuid) -> Result<(), String> {
        let mut queue_manager = self.queue_manager.clone();
        
        // 从等候区移除
        {
            let mut waiting_queue = queue_manager.waiting_queue.write().await;
            if let Some(pos) = waiting_queue.iter().position(|r| r.id == request_id) {
                waiting_queue.remove(pos);
                println!("从等候区移除请求: {}", request_id);
                return Ok(());
            }
        }
        
        // 从充电桩队列中移除
        {
            let mut pile_infos = queue_manager.pile_infos.write().await;
            for pile_info in pile_infos.values_mut() {
                // 检查当前充电的车辆
                if let Some(ref current) = pile_info.current_charging {
                    if current.id == request_id {
                        pile_info.current_charging = None;
                        pile_info.charging_start_time = None;
                        println!("取消当前充电请求: {}", request_id);
                        // 立即开始下一辆车充电
                        pile_info.start_next_charging(Utc::now()).await;
                        return Ok(());
                    }
                }
                
                // 检查队列中的车辆
                if let Some(pos) = pile_info.queue.iter().position(|r| r.id == request_id) {
                    pile_info.queue.remove(pos);
                    println!("从充电桩队列移除请求: {}", request_id);
                    return Ok(());
                }
            }
        }
        
        Err("未找到指定的充电请求".to_string())
    }

    /// 更新充电请求的充电量
    pub async fn update_request_amount(&self, request_id: Uuid, new_amount: f64) -> Result<(), String> {
        let mut queue_manager = self.queue_manager.clone();
        
        // 在等候区查找并更新
        {
            let mut waiting_queue = queue_manager.waiting_queue.write().await;
            for request in waiting_queue.iter_mut() {
                if request.id == request_id {
                    // 创建新的请求对象，因为Arc<ChargingRequest>是不可变的
                    let mut updated_request = (**request).clone();
                    updated_request.amount = new_amount;
                    updated_request.updated_at = Utc::now();
                    
                    // 替换原来的请求
                    *request = Arc::new(updated_request);
                    println!("✅ 更新等候区中请求 {} 的充电量为 {}度", request_id, new_amount);
                    return Ok(());
                }
            }
        }
        
        // 在充电桩队列中查找并更新
        {
            let mut pile_infos = queue_manager.pile_infos.write().await;
            for pile_info in pile_infos.values_mut() {
                // 检查当前充电的请求
                if let Some(ref current) = pile_info.current_charging {
                    if current.id == request_id {
                        return Err("充电中的请求不能修改充电量".to_string());
                    }
                }
                
                // 检查队列中的请求
                for request in pile_info.queue.iter_mut() {
                    if request.id == request_id {
                        let mut updated_request = (**request).clone();
                        updated_request.amount = new_amount;
                        updated_request.updated_at = Utc::now();
                        
                        *request = Arc::new(updated_request);
                        println!("✅ 更新充电桩队列中请求 {} 的充电量为 {}度", request_id, new_amount);
                        return Ok(());
                    }
                }
            }
        }
        
        Err("未找到指定的充电请求".to_string())
    }

    /// 更新充电请求的模式（需要重新排队）
    pub async fn update_request_mode(&self, request_id: Uuid, new_mode: ChargingMode, new_queue_number: String) -> Result<(), String> {
        let mut queue_manager = self.queue_manager.clone();
        
        // 先从原位置移除请求
        let mut found_request: Option<Arc<ChargingRequest>> = None;
        
        // 从等候区移除
        {
            let mut waiting_queue = queue_manager.waiting_queue.write().await;
            if let Some(pos) = waiting_queue.iter().position(|r| r.id == request_id) {
                found_request = Some(waiting_queue.remove(pos).unwrap());
                println!("从等候区移除请求: {}", request_id);
            }
        }
        
        // 从充电桩队列中移除
        if found_request.is_none() {
            let mut pile_infos = queue_manager.pile_infos.write().await;
            for pile_info in pile_infos.values_mut() {
                // 检查当前充电的请求
                if let Some(ref current) = pile_info.current_charging {
                    if current.id == request_id {
                        return Err("充电中的请求不能修改模式".to_string());
                    }
                }
                
                // 检查队列中的请求
                if let Some(pos) = pile_info.queue.iter().position(|r| r.id == request_id) {
                    found_request = Some(pile_info.queue.remove(pos).unwrap());
                    println!("从充电桩队列移除请求: {}", request_id);
                    break;
                }
            }
        }
        
        // 如果找到了请求，更新模式并重新提交
        if let Some(request) = found_request {
            let mut updated_request = (*request).clone();
            updated_request.mode = new_mode.to_string();
            updated_request.queue_number = new_queue_number;
            updated_request.updated_at = Utc::now();
            
            // 重新提交到等候区
            queue_manager.add_to_waiting_queue(Arc::new(updated_request)).await?;
            println!("✅ 请求 {} 已更新模式并重新排队", request_id);
            Ok(())
        } else {
            Err("未找到指定的充电请求".to_string())
        }
    }

    /// 通过用户ID取消充电请求
    pub async fn cancel_request_by_user(&self, user_id: Uuid) -> Result<(), String> {
        let mut queue_manager = self.queue_manager.clone();
        let mut found = false;
        
        // 从等候区移除该用户的所有请求
        {
            let mut waiting_queue = queue_manager.waiting_queue.write().await;
            let original_len = waiting_queue.len();
            waiting_queue.retain(|r| r.user_id != user_id);
            let removed_count = original_len - waiting_queue.len();
            if removed_count > 0 {
                println!("从等候区移除用户 {} 的 {} 个请求", user_id, removed_count);
                found = true;
            }
        }
        
        // 从充电桩队列中移除该用户的所有请求
        {
            let mut pile_infos = queue_manager.pile_infos.write().await;
            for pile_info in pile_infos.values_mut() {
                // 检查当前充电的车辆
                if let Some(ref current) = pile_info.current_charging {
                    if current.user_id == user_id {
                        println!("取消用户 {} 的当前充电请求: {}", user_id, current.id);
                        pile_info.current_charging = None;
                        pile_info.charging_start_time = None;
                        found = true;
                        // 立即开始下一辆车充电
                        pile_info.start_next_charging(Utc::now()).await;
                    }
                }
                
                // 检查队列中的车辆
                let original_len = pile_info.queue.len();
                pile_info.queue.retain(|r| r.user_id != user_id);
                let removed_count = original_len - pile_info.queue.len();
                if removed_count > 0 {
                    println!("从充电桩队列移除用户 {} 的 {} 个请求", user_id, removed_count);
                    found = true;
                }
            }
        }
        
        if found {
            Ok(())
        } else {
            Err("未找到该用户的充电请求".to_string())
        }
    }
}

/// 调度器状态
#[derive(Debug, Serialize)]
pub struct SchedulerStatus {
    pub is_running: bool,
    pub is_calling: bool,
}

/// 前端请求结构
#[derive(Debug, Deserialize)]
pub struct ChargingRequestInput {
    pub user_id: Uuid,
    pub mode: String, // "Fast" or "Slow"
    pub amount: f64,
}

/// 前端响应结构
#[derive(Debug, Serialize)]
pub struct ChargingRequestResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub mode: String,
    pub amount: f64,
    pub queue_number: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<ChargingRequest> for ChargingRequestResponse {
    fn from(request: ChargingRequest) -> Self {
        Self {
            id: request.id,
            user_id: request.user_id,
            mode: request.mode,
            amount: request.amount,
            queue_number: request.queue_number,
            status: request.status,
            created_at: request.created_at,
        }
    }
}

/// 全局调度器实例（用于Web接口）
use std::sync::OnceLock;
static GLOBAL_SCHEDULER: OnceLock<Arc<ChargingScheduler>> = OnceLock::new();

pub fn get_global_scheduler() -> Arc<ChargingScheduler> {
    GLOBAL_SCHEDULER.get_or_init(|| Arc::new(ChargingScheduler::new())).clone()
}

/// 初始化全局调度器并设置数据库连接池
pub fn init_global_scheduler_with_db(db_pool: Arc<sqlx::MySqlPool>) -> Arc<ChargingScheduler> {
    GLOBAL_SCHEDULER.get_or_init(|| {
        Arc::new(ChargingScheduler::new().with_db_pool(db_pool))
    }).clone()
}

/// 初始化并启动全局调度器
pub async fn init_global_scheduler() -> Result<(), String> {
    let scheduler = get_global_scheduler();
    scheduler.start().await
}

/// 停止全局调度器
pub async fn stop_global_scheduler() -> Result<(), String> {
    let scheduler = get_global_scheduler();
    scheduler.stop().await
}

// ==== 前端接口函数 ====

/// 提交充电请求接口
pub async fn api_submit_request(input: ChargingRequestInput) -> Result<ChargingRequestResponse, String> {
    let mode = match input.mode.as_str() {
        "Fast" => ChargingMode::Fast,
        "Slow" => ChargingMode::Slow,
        _ => return Err("无效的充电模式".to_string()),
    };

    let scheduler = get_global_scheduler();
    let request = ChargingRequest::new(input.user_id, mode, input.amount, "".to_string());
    scheduler.submit_request(request.clone()).await?;
    
    Ok(ChargingRequestResponse::from(request))
}

/// 获取系统状态接口
pub async fn api_get_system_status() -> SystemStatus {
    let scheduler = get_global_scheduler();
    scheduler.get_system_status().await
}

/// 获取调度器状态接口
pub async fn api_get_scheduler_status() -> SchedulerStatus {
    let scheduler = get_global_scheduler();
    scheduler.get_scheduler_status().await
}

/// 充电桩故障处理接口
pub async fn api_handle_pile_fault(pile_id: String) -> Result<(), String> {
    let scheduler = get_global_scheduler();
    scheduler.handle_pile_fault(&pile_id).await
}

/// 充电桩恢复处理接口
pub async fn api_handle_pile_recovery(pile_id: String) -> Result<(), String> {
    let scheduler = get_global_scheduler();
    scheduler.handle_pile_recovery(&pile_id).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub pile_statuses: Vec<PileStatus>,
    pub fast_waiting_count: usize,
    pub slow_waiting_count: usize,
    pub fast_waiting_requests: Vec<ChargingRequest>,
    pub slow_waiting_requests: Vec<ChargingRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PileStatus {
    pub pile_number: String,
    pub pile_mode: ChargingMode,
    pub is_idle: bool,
    pub current_charging_user: Option<Uuid>,
    pub current_request: Option<ChargingRequest>,
    pub queue_count: usize,
    pub queue_requests: Vec<ChargingRequest>,
    pub charging_progress: Option<f64>,
}