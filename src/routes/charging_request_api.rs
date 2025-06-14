use std::str::FromStr;
use std::sync::Arc;

use charging_station::models::{ChargingMode, ChargingRequest, RequestStatus};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::Utc;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::sync::Mutex;
use uuid::Uuid;

// 请求结构体
#[derive(Debug, Deserialize)]
pub struct CreateChargingRequestPayload {
    pub user_id: Uuid,
    pub mode: ChargingMode,
    pub amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChargingModePayload {
    pub mode: ChargingMode,
    pub queue_number: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChargingAmountPayload {
    pub amount: f64,
}

// 响应结构体
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}

/// 生成排队号码
fn generate_queue_number(mode: ChargingMode, position: usize) -> String {
    match mode {
        ChargingMode::Fast => format!("F{}", position + 1),
        ChargingMode::Slow => format!("T{}", position + 1),
    }
}

// 创建充电请求（直接调用调度器）
#[post("/charging-requests")]
pub async fn create_charging_request(
    scheduler: web::Data<Arc<charging_station::scheduler::ChargingScheduler>>,
    payload: web::Json<CreateChargingRequestPayload>,
) -> impl Responder {
    let request = charging_station::models::ChargingRequest::new(
        payload.user_id,
        payload.mode,
        payload.amount,
        "".to_string(),
    );
    match scheduler.submit_request(request.clone()).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success(request, "充电请求创建成功")),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(&e)),
    }
}

/// 检查用户是否在等待区
async fn is_user_in_waiting_area(
    user_id: Uuid,
    scheduler: &web::Data<Arc<charging_station::scheduler::ChargingScheduler>>,
) -> bool {
    let system_status = scheduler.get_system_status().await;
    
    // 检查用户是否在等待区（快充或慢充等待区）
    let in_fast_waiting = system_status.fast_waiting_requests.iter().any(|req| req.user_id == user_id);
    let in_slow_waiting = system_status.slow_waiting_requests.iter().any(|req| req.user_id == user_id);
    
    // 确保用户不在任何充电桩的队列或充电中
    let not_in_pile = !system_status.pile_statuses.iter().any(|pile| {
        // 检查是否正在充电
        let is_charging = pile.current_request
            .as_ref()
            .map(|req| req.user_id == user_id)
            .unwrap_or(false);
            
        // 检查是否在充电桩队列中
        let in_queue = pile.queue_requests
            .iter()
            .any(|req| req.user_id == user_id);
            
        is_charging || in_queue
    });
    
    // 只有在等待区列表中，且不在任何充电桩的情况下，才返回true
    (in_fast_waiting || in_slow_waiting) && not_in_pile
}

// 修改充电模式（直接调用调度器）
#[put("/charging-requests/{id}/mode")]
pub async fn update_charging_mode(
    path: web::Path<Uuid>,
    payload: web::Json<UpdateChargingModePayload>,
    scheduler: web::Data<Arc<charging_station::scheduler::ChargingScheduler>>,
) -> impl Responder {
    let request_id = path.into_inner();
    let new_mode = payload.mode;
    let new_queue_number = payload.queue_number.clone();
    match scheduler.update_request_mode(request_id, new_mode, new_queue_number).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success((), "充电模式修改成功")),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(&e)),
    }
}

// 修改充电量（直接调用调度器）
#[put("/charging-requests/{id}/amount")]
pub async fn update_charging_amount(
    path: web::Path<Uuid>,
    payload: web::Json<UpdateChargingAmountPayload>,
    scheduler: web::Data<Arc<charging_station::scheduler::ChargingScheduler>>,
) -> impl Responder {
    let request_id = path.into_inner();
    let new_amount = payload.amount;
    match scheduler.update_request_amount(request_id, new_amount).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success((), "充电量修改成功")),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(&e)),
    }
}

// 取消充电请求（直接调用调度器）
#[delete("/charging-requests/{id}")]
pub async fn cancel_charging_request(
    scheduler: web::Data<Arc<charging_station::scheduler::ChargingScheduler>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let request_id = path.into_inner();
    match scheduler.cancel_request(request_id).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success((), "充电请求已取消")),
        Err(e) => HttpResponse::BadRequest().json(ApiResponse::<()>::error(&e)),
    }
}

// 获取用户的所有充电请求（调度器队列+所有桩队列）
#[get("/users/{user_id}/charging-requests")]
pub async fn get_user_charging_requests(
    scheduler: web::Data<Arc<charging_station::scheduler::ChargingScheduler>>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let user_id = path.into_inner();
    // 汇总等候区和所有桩队列的请求
    let mut result = Vec::new();
    let queue_manager = &scheduler.queue_manager;
    let waiting_queue = queue_manager.waiting_queue.read().await;
    for req in waiting_queue.iter() {
        if req.user_id == user_id {
            result.push((**req).clone());
        }
    }
    let pile_infos = queue_manager.pile_infos.read().await;
    for pile_info in pile_infos.values() {
        if let Some(ref current) = pile_info.current_charging {
            if current.user_id == user_id {
                result.push((**current).clone());
            }
        }
        for req in pile_info.queue.iter() {
            if req.user_id == user_id {
                result.push((**req).clone());
            }
        }
    }
    HttpResponse::Ok().json(ApiResponse::success(result, "获取用户充电请求成功"))
}

// 获取充电队列（按模式）
#[get("/charging-requests/queue/{mode}")]
pub async fn get_charging_queue(
    scheduler: web::Data<Arc<charging_station::scheduler::ChargingScheduler>>,
    path: web::Path<String>,
) -> impl Responder {
    let mode_str = path.into_inner();
    let mode = match mode_str.as_str() {
        "fast" | "Fast" => ChargingMode::Fast,
        "slow" | "Slow" => ChargingMode::Slow,
        _ => {
            return HttpResponse::BadRequest().json(ApiResponse::<()>::error("无效的充电模式"));
        }
    };
    let queue_manager = &scheduler.queue_manager;
    let waiting_queue = queue_manager.waiting_queue.read().await;
    let mut requests: Vec<_> = waiting_queue
        .iter()
        .filter(|req| req.mode == mode.to_string())
        .map(|req| (**req).clone())
        .collect();
    requests.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    HttpResponse::Ok().json(ApiResponse::success(requests, "获取充电队列成功"))
}

// 获取所有充电请求（管理员接口，等候区+所有桩队列）
#[get("/charging-requests")]
pub async fn get_all_charging_requests(
    scheduler: web::Data<Arc<charging_station::scheduler::ChargingScheduler>>,
) -> impl Responder {
    let mut result = Vec::new();
    let queue_manager = &scheduler.queue_manager;
    let waiting_queue = queue_manager.waiting_queue.read().await;
    for req in waiting_queue.iter() {
        result.push((**req).clone());
    }
    let pile_infos = queue_manager.pile_infos.read().await;
    for pile_info in pile_infos.values() {
        if let Some(ref current) = pile_info.current_charging {
            result.push((**current).clone());
        }
        for req in pile_info.queue.iter() {
            result.push((**req).clone());
        }
    }
    HttpResponse::Ok().json(ApiResponse::success(result, "获取所有充电请求成功"))
}

// 配置路由
pub fn charging_request_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_charging_request)
        .service(update_charging_mode)
        .service(update_charging_amount)
        .service(cancel_charging_request)
        .service(get_user_charging_requests)
        .service(get_charging_queue)
        .service(get_all_charging_requests);
}