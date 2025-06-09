use std::str::FromStr;

use crate::models::{ChargingMode, ChargingRequest, RequestStatus};
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
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

/// 创建充电请求
#[post("/charging-requests")]
pub async fn create_charging_request(
    pool: web::Data<MySqlPool>,
    payload: web::Json<CreateChargingRequestPayload>,
) -> impl Responder {
    // 获取当前队列长度来生成排队号码
    let waiting_requests = match ChargingRequest::get_queue(
        pool.get_ref(),
        payload.mode,
        RequestStatus::Waiting,
    )
    .await
    {
        Ok(requests) => requests,
        Err(err) => {
            eprintln!("Error fetching queue: {:?}", err);
            eprintln!("⚠️ SQL执行失败: {:?}", err);
            eprintln!(
                "⚠️ mode = {:?}, as string = {}",
                payload.mode,
                payload.mode.to_string()
            );
            eprintln!("⚠️ status = {:?}", RequestStatus::Waiting);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("获取队列信息失败"));
        }
    };

    let queue_number = generate_queue_number(payload.mode, waiting_requests.len());

    let mut request =
        ChargingRequest::new(payload.user_id, payload.mode, payload.amount, queue_number);

    match request.create(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::success(request, "充电请求创建成功")),
        Err(err) => {
            eprintln!("Error creating charging request: {:?}", err);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("创建充电请求失败"))
        }
    }
}

/// 修改充电模式
#[put("/charging-requests/{id}/mode")]
pub async fn update_charging_mode(
    pool: web::Data<MySqlPool>,
    path: web::Path<Uuid>,
    payload: web::Json<UpdateChargingModePayload>,
) -> impl Responder {
    let request_id = path.into_inner();

    let mut request = match ChargingRequest::get_by_id(pool.get_ref(), request_id).await {
        Ok(Some(request)) => request,
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error("充电请求不存在"));
        }
        Err(err) => {
            eprintln!("Error fetching charging request: {:?}", err);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("获取充电请求失败"));
        }
    };

    // 检查请求状态
    let status = match RequestStatus::from_str(&request.status) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(&format!("状态转换错误: {}", e)));
        }
    };

    match status {
        RequestStatus::Waiting => {
            // 等候区允许修改，需要重新生成排队号并排到队列最后
            let waiting_requests = match ChargingRequest::get_queue(
                pool.get_ref(),
                payload.mode,
                RequestStatus::Waiting,
            )
            .await
            {
                Ok(requests) => requests,
                Err(err) => {
                    eprintln!("Error fetching queue: {:?}", err);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("获取队列信息失败"));
                }
            };

            let new_queue_number = generate_queue_number(payload.mode, waiting_requests.len());
            request.update_mode(payload.mode, new_queue_number);

            match request.update(pool.get_ref()).await {
                Ok(_) => HttpResponse::Ok().json(ApiResponse::success(
                    request,
                    "充电模式修改成功，已重新排队",
                )),
                Err(err) => {
                    eprintln!("Error updating charging mode: {:?}", err);
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("修改充电模式失败"))
                }
            }
        }
        RequestStatus::Charging => HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "充电中不允许修改模式，请先取消充电",
        )),
        _ => HttpResponse::BadRequest().json(ApiResponse::<()>::error("当前状态不允许修改模式")),
    }
}

/// 修改请求充电量
#[put("/charging-requests/{id}/amount")]
pub async fn update_charging_amount(
    pool: web::Data<MySqlPool>,
    path: web::Path<Uuid>,
    payload: web::Json<UpdateChargingAmountPayload>,
) -> impl Responder {
    let request_id = path.into_inner();

    let mut request = match ChargingRequest::get_by_id(pool.get_ref(), request_id).await {
        Ok(Some(request)) => request,
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error("充电请求不存在"));
        }
        Err(err) => {
            eprintln!("Error fetching charging request: {:?}", err);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("获取充电请求失败"));
        }
    };

    let status = match RequestStatus::from_str(&request.status) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(&format!("状态转换错误: {}", e)));
        }
    };

    // 检查请求状态
    match status {
        RequestStatus::Waiting => {
            // 等候区允许修改充电量，排队号不变
            request.update_amount(payload.amount);

            match request.update(pool.get_ref()).await {
                Ok(_) => HttpResponse::Ok().json(ApiResponse::success(request, "充电量修改成功")),
                Err(err) => {
                    eprintln!("Error updating charging amount: {:?}", err);
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("修改充电量失败"))
                }
            }
        }
        RequestStatus::Charging => HttpResponse::BadRequest().json(ApiResponse::<()>::error(
            "充电中不允许修改充电量，请先取消充电",
        )),
        _ => HttpResponse::BadRequest().json(ApiResponse::<()>::error("当前状态不允许修改充电量")),
    }
}

/// 取消充电请求
#[delete("/charging-requests/{id}")]
pub async fn cancel_charging_request(
    pool: web::Data<MySqlPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let request_id = path.into_inner();

    let mut request = match ChargingRequest::get_by_id(pool.get_ref(), request_id).await {
        Ok(Some(request)) => request,
        Ok(None) => {
            return HttpResponse::NotFound().json(ApiResponse::<()>::error("充电请求不存在"));
        }
        Err(err) => {
            eprintln!("Error fetching charging request: {:?}", err);
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("获取充电请求失败"));
        }
    };

    // 等候区和充电区均允许取消
    let status = match RequestStatus::from_str(&request.status) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error(&format!("状态转换错误: {}", e)));
        }
    };

    match status {
        RequestStatus::Waiting | RequestStatus::Charging => {
            if let Err(err) = request.cancel() {
                return HttpResponse::BadRequest().json(ApiResponse::<()>::error(&err));
            }

            match request
                .update_status(pool.get_ref(), RequestStatus::Cancelled)
                .await
            {
                Ok(_) => HttpResponse::Ok().json(ApiResponse::success(request, "充电请求已取消")),
                Err(err) => {
                    eprintln!("Error cancelling charging request: {:?}", err);
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("取消充电请求失败"))
                }
            }
        }
        _ => HttpResponse::BadRequest().json(ApiResponse::<()>::error("当前状态不允许取消")),
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
    pool: web::Data<MySqlPool>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let user_id = path.into_inner();

    match ChargingRequest::get_by_user_id(pool.get_ref(), user_id).await {
        Ok(requests) => {
            HttpResponse::Ok().json(ApiResponse::success(requests, "获取用户充电请求成功"))
        }
        Err(err) => {
            eprintln!("Error fetching user charging requests: {:?}", err);
            HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("获取用户充电请求失败"))
        }
    }
}

/// 获取充电队列
#[get("/charging-requests/queue/{mode}")]
pub async fn get_charging_queue(
    pool: web::Data<MySqlPool>,
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

    match ChargingRequest::get_queue(pool.get_ref(), mode, RequestStatus::Waiting).await {
        Ok(requests) => HttpResponse::Ok().json(ApiResponse::success(requests, "获取充电队列成功")),
        Err(err) => {
            eprintln!("Error fetching charging queue: {:?}", err);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("获取充电队列失败"))
        }
    }
}

/// 获取所有充电请求（管理员接口）
#[get("/charging-requests")]
pub async fn get_all_charging_requests(pool: web::Data<MySqlPool>) -> impl Responder {
    match ChargingRequest::get_by_status(pool.get_ref(), RequestStatus::Waiting).await {
        Ok(waiting_requests) => {
            match ChargingRequest::get_by_status(pool.get_ref(), RequestStatus::Charging).await {
                Ok(charging_requests) => {
                    let mut all_requests = waiting_requests;
                    all_requests.extend(charging_requests);

                    HttpResponse::Ok()
                        .json(ApiResponse::success(all_requests, "获取所有充电请求成功"))
                }
                Err(err) => {
                    eprintln!("Error fetching charging requests: {:?}", err);
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("获取充电请求失败"))
                }
            }
        }
        Err(err) => {
            eprintln!("Error fetching waiting requests: {:?}", err);
            HttpResponse::InternalServerError().json(ApiResponse::<()>::error("获取充电请求失败"))
        }
    }
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
