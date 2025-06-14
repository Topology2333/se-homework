use crate::models::charging_pile::{ChargingPile, PileStatus};
use crate::models::ChargingRequest;
use actix_web::{get, post, web, HttpResponse, Responder};
use sqlx::MySqlPool;
use uuid::Uuid;

#[get("/piles")]
pub async fn get_all_piles(pool: web::Data<MySqlPool>) -> impl Responder {
    match ChargingPile::get_all(pool.get_ref()).await {
        Ok(piles) => HttpResponse::Ok().json(piles),
        Err(err) => {
            eprintln!("Error fetching piles: {:?}", err); // 打印错误
            HttpResponse::InternalServerError().body("获取充电桩信息失败")
        }
    }
}

#[post("/piles/{id}/start")]
pub async fn start_pile(pool: web::Data<MySqlPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let mut piles = ChargingPile::get_all(pool.get_ref())
        .await
        .unwrap_or_default();

    if let Some(pile) = piles.iter_mut().find(|p| p.id == id) {
        // 检查充电桩状态，只有在空闲或关闭状态下才可以启动
        if pile.status == PileStatus::Shutdown {
            pile.status = PileStatus::Available;
            pile.started_at = Some(chrono::Utc::now());
            let _ = pile.update_status(pool.get_ref()).await;
            HttpResponse::Ok().body("充电桩已启动")
        } else if pile.status == PileStatus::Available {
            HttpResponse::BadRequest().body("充电桩已启动")
        } else {
            HttpResponse::BadRequest().body("充电桩正在充电")
        }
    } else {
        HttpResponse::NotFound().body("未找到充电桩")
    }
}

#[post("/piles/{id}/shutdown")]
pub async fn shutdown_pile(pool: web::Data<MySqlPool>, path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    let mut piles = ChargingPile::get_all(pool.get_ref())
        .await
        .unwrap_or_default();

    if let Some(pile) = piles.iter_mut().find(|p| p.id == id) {
        // 检查充电桩状态，只有在空闲或关闭状态下才可以关闭
        if pile.status == PileStatus::Available {
            pile.status = PileStatus::Shutdown;
            pile.started_at = None;
            let _ = pile.update_status(pool.get_ref()).await;
            HttpResponse::Ok().body("充电桩已关闭")
        } else if pile.status == PileStatus::Charging {
            HttpResponse::BadRequest().body("充电桩正在充电，无法关闭")
        } else {
            HttpResponse::BadRequest().body("充电桩已经关闭")
        }
    } else {
        HttpResponse::NotFound().body("未找到充电桩")
    }
}

// #[get("/piles/{id}/waiting-requests")]
// pub async fn get_waiting_requests(
//     pool: web::Data<MySqlPool>,
//     path: web::Path<Uuid>,
// ) -> impl Responder {
//     match ChargingRequest::get_waiting_requests(pool.get_ref(), path.into_inner()).await {
//         Ok(requests) => HttpResponse::Ok().json(requests),
//         Err(err) => {
//             eprintln!("Error fetching waiting requests: {:?}", err);
//             HttpResponse::InternalServerError().body("获取等待队列失败")
//         }
//     }
// }

// 修改 pile_routes 函数添加新路由
pub fn pile_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_all_piles)
        .service(start_pile)
        .service(shutdown_pile);
        // .service(get_waiting_requests);
}
