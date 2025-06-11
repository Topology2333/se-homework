use std::str::FromStr;

use crate::models::{ChargingMode, ChargingRequest, RequestStatus};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use chrono::Utc;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use std::sync::Mutex;
use uuid::Uuid;

// 创建静态充电请求列表
lazy_static! {
    static ref CHARGING_REQUESTS: Mutex<Vec<ChargingRequest>> = Mutex::new(vec![]);
}

// 添加一个初始化函数
pub fn init_charging_requests() {
    let mut requests = CHARGING_REQUESTS.lock().unwrap();
    requests.clear();
}

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

/// 创建充电请求
#[post("/charging-requests")]
pub async fn create_charging_request(
    _pool: web::Data<MySqlPool>,
    payload: web::Json<CreateChargingRequestPayload>,
) -> impl Responder {
    let mut requests = CHARGING_REQUESTS.lock().unwrap();

    // 获取当前等待数量
    let waiting_count = requests
        .iter()
        .filter(|r| {
            r.mode == payload.mode
                && RequestStatus::from_str(&r.status).unwrap() == RequestStatus::Waiting
        })
        .count();

    // 使用generate_queue_number生成排队号码
    let queue_number = generate_queue_number(payload.mode, waiting_count);

    let mut request =
        ChargingRequest::new(payload.user_id, payload.mode, payload.amount, queue_number);

    // 将新请求添加到静态列表中
    requests.push(request.clone());

    HttpResponse::Ok().json(ApiResponse::success(request, "充电请求创建成功"))
}

/// 修改充电模式
#[put("/charging-requests/{id}/mode")]
pub async fn update_charging_mode(
    path: web::Path<Uuid>,
    payload: web::Json<UpdateChargingModePayload>,
) -> impl Responder {
    let request_id = path.into_inner();
    let mut requests = CHARGING_REQUESTS.lock().unwrap();
    let request_index = requests.iter().position(|r| r.id == request_id);
    if let Some(idx) = request_index {
        // 先不可变借用统计 waiting_count
        let waiting_count = requests
            .iter()
            .filter(|r| {
                r.mode == payload.mode
                    && RequestStatus::from_str(&r.status).unwrap() == RequestStatus::Waiting
            })
            .count();
        // 再可变借用
        let request = &mut requests[idx];
        match RequestStatus::from_str(&request.status).unwrap() {
            RequestStatus::Waiting => {
                let new_queue_number = generate_queue_number(payload.mode, waiting_count);
                request.mode = payload.mode.to_string();
                request.queue_number = new_queue_number;
                request.created_at = Utc::now(); // 更新修改时间
                return HttpResponse::Ok().json(ApiResponse::success(
                    request.clone(),
                    "充电模式修改成功，已重新排队",
                ));
            }
            RequestStatus::Charging => {
                return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                    "充电中不允许修改模式，请先取消充电",
                ));
            }
            _ => {
                return HttpResponse::BadRequest()
                    .json(ApiResponse::<()>::error("当前状态不允许修改模式"));
            }
        }
    } else {
        return HttpResponse::NotFound().json(ApiResponse::<()>::error("充电请求不存在"));
    }
}

/// 修改请求充电量
#[put("/charging-requests/{id}/amount")]
pub async fn update_charging_amount(
    path: web::Path<Uuid>,
    payload: web::Json<UpdateChargingAmountPayload>,
) -> impl Responder {
    let request_id = path.into_inner();

    let mut requests = CHARGING_REQUESTS.lock().unwrap();
    if let Some(request) = requests.iter_mut().find(|r| r.id == request_id) {
        match RequestStatus::from_str(&request.status).unwrap() {
            RequestStatus::Waiting => {
                request.amount = payload.amount;
                return HttpResponse::Ok()
                    .json(ApiResponse::success(request.clone(), "充电量修改成功"));
            }
            RequestStatus::Charging => {
                return HttpResponse::BadRequest().json(ApiResponse::<()>::error(
                    "充电中不允许修改充电量，请先取消充电",
                ));
            }
            _ => {
                return HttpResponse::BadRequest()
                    .json(ApiResponse::<()>::error("当前状态不允许修改充电量"));
            }
        }
    } else {
        return HttpResponse::NotFound().json(ApiResponse::<()>::error("充电请求不存在"));
    }
}

/// 取消充电请求
#[delete("/charging-requests/{id}")]
pub async fn cancel_charging_request(
    _pool: web::Data<MySqlPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let request_id = path.into_inner();

    let mut requests = CHARGING_REQUESTS.lock().unwrap();

    // 找到并删除对应的请求
    if let Some(index) = requests.iter().position(|req| req.id == request_id) {
        requests.remove(index);
        HttpResponse::Ok().json(ApiResponse::success((), "充电请求已取消"))
    } else {
        HttpResponse::NotFound().json(ApiResponse::<()>::error("充电请求不存在"))
    }
}

/// 获取单个充电请求
#[get("/charging-requests/{id}")]
pub async fn get_charging_request(
    pool: web::Data<MySqlPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let request_id = path.into_inner();

    match ChargingRequest::get_by_id(pool.get_ref(), request_id).await {
        Ok(Some(request)) => {
            HttpResponse::Ok().json(ApiResponse::success(request, "获取充电请求成功"))
        }
        Ok(None) => HttpResponse::NotFound().json(ApiResponse::<()>::error("充电请求不存在")),
        Err(err) => {
            eprintln!("Error fetching charging request: {:?}", err);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("获取充电请求失败"))
        }
    }
}

/// 获取用户的所有充电请求
#[get("/users/{user_id}/charging-requests")]
pub async fn get_user_charging_requests(
    _pool: web::Data<MySqlPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let user_id = path.into_inner();

    // 过滤出当前用户的充电请求
    let requests = CHARGING_REQUESTS
        .lock()
        .unwrap()
        .iter()
        .filter(|req| req.user_id == user_id)
        .cloned()
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(ApiResponse::success(requests, "获取用户充电请求成功"))
}

/// 获取充电队列
#[get("/charging-requests/queue/{mode}")]
pub async fn get_charging_queue(
    _pool: web::Data<MySqlPool>,
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

    // 从静态列表中获取指定模式的等待中请求，并按创建时间排序
    let mut requests = CHARGING_REQUESTS
        .lock()
        .unwrap()
        .iter()
        .filter(|req| {
            req.mode == mode
                && RequestStatus::from_str(&req.status).unwrap() == RequestStatus::Waiting
        })
        .cloned()
        .collect::<Vec<_>>();

    // 按创建时间排序
    requests.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    HttpResponse::Ok().json(ApiResponse::success(requests, "获取充电队列成功"))
}

/// 获取所有充电请求（管理员接口）
#[get("/charging-requests")]
pub async fn get_all_charging_requests(_pool: web::Data<MySqlPool>) -> impl Responder {
    // 从静态列表中获取所有请求
    let requests = CHARGING_REQUESTS
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    HttpResponse::Ok().json(ApiResponse::success(requests, "获取所有充电请求成功"))
}

// 配置路由
pub fn charging_request_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_charging_request)
        .service(update_charging_mode)
        .service(update_charging_amount)
        .service(cancel_charging_request)
        .service(get_charging_request)
        .service(get_user_charging_requests)
        .service(get_charging_queue)
        .service(get_all_charging_requests);
}
