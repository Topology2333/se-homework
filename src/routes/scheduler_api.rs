use actix_web::{web, HttpResponse, Responder};
use charging_station::scheduler::ChargingScheduler;
use charging_station::models::{ChargingRequest, ChargingMode, RequestStatus};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use chrono::{Utc};
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct SystemStatus {
    pub is_running: bool,
    pub waiting_count: usize,
    pub charging_count: usize,
    pub pile_status: Vec<PileStatus>,
}

#[derive(Debug, Serialize)]
pub struct PileStatus {
    pub number: String,
    pub mode: String,
    pub status: String,
    pub current_user: Option<String>,
    pub queue_count: usize,
    pub charging_progress: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct StartChargingRequest {
    pub user_id: Uuid,
    pub mode: String,
    pub amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChargingRequest {
    pub request_id: Uuid,
    pub mode: String,
    pub amount: f64,
}

pub async fn get_system_status(scheduler: web::Data<Arc<ChargingScheduler>>) -> impl Responder {
    let status = scheduler.get_system_status().await;
    HttpResponse::Ok().json(json!({
        "is_running": scheduler.get_scheduler_status().await.is_running,
        "waiting_count": status.fast_waiting_count + status.slow_waiting_count,
        "charging_count": status.pile_statuses.iter().filter(|pile| !pile.is_idle).count(),
        "pile_status": status.pile_statuses,
        "fast_waiting_requests": status.fast_waiting_requests,
        "slow_waiting_requests": status.slow_waiting_requests
    }))
}

pub async fn start_scheduler(scheduler: web::Data<Arc<ChargingScheduler>>) -> impl Responder {
    match scheduler.start().await {
        Ok(_) => HttpResponse::Ok().json("调度系统已启动"),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

pub async fn stop_scheduler(scheduler: web::Data<Arc<ChargingScheduler>>) -> impl Responder {
    match scheduler.stop().await {
        Ok(_) => HttpResponse::Ok().json("调度系统已停止"),
        Err(e) => HttpResponse::InternalServerError().json(e),
    }
}

pub async fn submit_charging_request(
    scheduler: web::Data<Arc<ChargingScheduler>>,
    request: web::Json<StartChargingRequest>,
) -> impl Responder {
    let mode = match request.mode.as_str() {
        "Fast" => ChargingMode::Fast,
        "Slow" => ChargingMode::Slow,
        _ => return HttpResponse::BadRequest().json("无效的充电模式"),
    };

    let charging_request = ChargingRequest {
        id: Uuid::new_v4(),
        user_id: request.user_id,
        mode: mode.to_string(),
        amount: request.amount,
        queue_number: String::new(),
        status: RequestStatus::Waiting.to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    match scheduler.submit_request(charging_request.clone()).await {
        Ok(_) => HttpResponse::Ok().json(charging_request),
        Err(e) => HttpResponse::BadRequest().json(e),
    }
}

pub async fn get_pile_status(scheduler: web::Data<Arc<ChargingScheduler>>) -> impl Responder {
    let status = scheduler.get_system_status().await;
    let pile_status: Vec<PileStatus> = status.pile_statuses.into_iter().map(|info| {
        PileStatus {
            number: info.pile_number,
            mode: info.pile_mode.to_string(),
            status: if info.is_idle { "空闲".to_string() } else { "充电中".to_string() },
            current_user: info.current_charging_user.map(|id| id.to_string()),
            queue_count: info.queue_count,
            charging_progress: info.charging_progress,
        }
    }).collect();

    HttpResponse::Ok().json(pile_status)
}

pub async fn get_waiting_queue(scheduler: web::Data<Arc<ChargingScheduler>>) -> impl Responder {
    let status = scheduler.get_system_status().await;
    HttpResponse::Ok().json(status)
}

/// 取消充电请求
pub async fn cancel_charging_request(
    scheduler: web::Data<Arc<ChargingScheduler>>,
    request_id: web::Path<Uuid>,
) -> impl Responder {
    match scheduler.cancel_request(request_id.into_inner()).await {
        Ok(_) => HttpResponse::Ok().json(json!({
            "message": "请求已取消",
            "success": true
        })),
        Err(e) => {
            println!("取消请求失败: {}", e);
            HttpResponse::BadRequest().json(json!({
                "message": e,
                "success": false
            }))
        }
    }
}

/// 通过用户ID取消充电请求
pub async fn cancel_charging_request_by_user(
    scheduler: web::Data<Arc<ChargingScheduler>>,
    user_id: web::Path<Uuid>,
) -> impl Responder {
    let user_id = user_id.into_inner();
    println!("收到取消用户 {} 的充电请求", user_id);
    
    match scheduler.cancel_request_by_user(user_id).await {
        Ok(_) => {
            println!("成功取消用户 {} 的充电请求", user_id);
            HttpResponse::Ok().json(json!({
                "message": "用户请求已取消",
                "success": true
            }))
        },
        Err(e) => {
            println!("取消用户 {} 的请求失败: {}", user_id, e);
            HttpResponse::BadRequest().json(json!({
                "message": e,
                "success": false
            }))
        }
    }
}

/// 更新充电请求的充电量
pub async fn update_charging_amount(
    scheduler: web::Data<Arc<ChargingScheduler>>,
    path: web::Path<Uuid>,
    request: web::Json<serde_json::Value>,
) -> impl Responder {
    let request_id = path.into_inner();
    
    if let Some(amount) = request.get("amount").and_then(|v| v.as_f64()) {
        match scheduler.update_request_amount(request_id, amount).await {
            Ok(_) => HttpResponse::Ok().json(json!({
                "message": "充电量更新成功",
                "success": true
            })),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "message": e,
                "success": false
            }))
        }
    } else {
        HttpResponse::BadRequest().json(json!({
            "message": "无效的充电量参数",
            "success": false
        }))
    }
}

/// 更新充电请求的模式
pub async fn update_charging_mode(
    scheduler: web::Data<Arc<ChargingScheduler>>,
    path: web::Path<Uuid>,
    request: web::Json<serde_json::Value>,
) -> impl Responder {
    let request_id = path.into_inner();
    
    if let (Some(mode_str), Some(queue_number)) = (
        request.get("mode").and_then(|v| v.as_str()),
        request.get("queue_number").and_then(|v| v.as_str())
    ) {
        let mode = match mode_str {
            "Fast" => ChargingMode::Fast,
            "Slow" => ChargingMode::Slow,
            _ => return HttpResponse::BadRequest().json(json!({
                "message": "无效的充电模式",
                "success": false
            }))
        };
        
        match scheduler.update_request_mode(request_id, mode, queue_number.to_string()).await {
            Ok(_) => HttpResponse::Ok().json(json!({
                "message": "充电模式更新成功，已重新排队",
                "success": true
            })),
            Err(e) => HttpResponse::BadRequest().json(json!({
                "message": e,
                "success": false
            }))
        }
    } else {
        HttpResponse::BadRequest().json(json!({
            "message": "无效的参数",
            "success": false
        }))
    }
}

/// 更新充电请求（已废弃，请使用分离的接口）
pub async fn update_charging_request(
    _scheduler: web::Data<Arc<ChargingScheduler>>,
    _request: web::Json<UpdateChargingRequest>,
) -> impl Responder {
    HttpResponse::BadRequest().json(json!({
        "message": "此接口已废弃，请使用 /api/charging-requests/{id}/amount 或 /api/charging-requests/{id}/mode 接口",
        "success": false
    }))
}

/// 测试充电完成（仅用于调试）
pub async fn test_charging_completion(
    scheduler: web::Data<Arc<ChargingScheduler>>,
) -> Result<HttpResponse, actix_web::Error> {
    println!("🧪 手动触发充电完成检查");
    
    // 手动调用tick来检查充电完成
    scheduler.manual_tick().await;
    
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "充电完成检查已触发"
    })))
}

/// 配置服务路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/scheduler")
            .route("/status", web::get().to(get_system_status))
            .route("/start", web::post().to(start_scheduler))
            .route("/stop", web::post().to(stop_scheduler))
            .route("/submit", web::post().to(submit_charging_request))
            .route("/piles", web::get().to(get_pile_status))
            .route("/waiting", web::get().to(get_waiting_queue))
            .route("/cancel/{request_id}", web::post().to(cancel_charging_request))
            .route("/cancel/user/{user_id}", web::post().to(cancel_charging_request_by_user))
            .route("/update", web::post().to(update_charging_request))
            .route("/update/{request_id}/amount", web::put().to(update_charging_amount))
            .route("/update/{request_id}/mode", web::put().to(update_charging_mode))
            .route("/test-completion", web::post().to(test_charging_completion))
    );
} 