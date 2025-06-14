use actix_web::{web, HttpResponse, Result};
use sqlx::MySqlPool;
use uuid::Uuid;
use crate::models::ChargingRecord;
use serde_json::json;

/// 根据用户ID获取充电详单
pub async fn get_user_charging_records(
    path: web::Path<Uuid>,
    pool: web::Data<MySqlPool>,
) -> Result<HttpResponse> {
    let user_id = path.into_inner();
    
    match ChargingRecord::find_by_user_id(user_id, &pool).await {
        Ok(records) => {
            println!("✅ 查询到用户 {} 的 {} 条充电详单", user_id, records.len());
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "data": records,
                "count": records.len()
            })))
        }
        Err(e) => {
            println!("❌ 查询充电详单失败: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": format!("查询充电详单失败: {}", e)
            })))
        }
    }
}

/// 测试插入充电详单（仅用于调试）
pub async fn test_insert_record(
    pool: web::Data<MySqlPool>,
) -> Result<HttpResponse, actix_web::Error> {
    use crate::models::ChargingMode;
    use chrono::Utc;
    
    // 创建一个测试充电详单
    let test_record = ChargingRecord::new(
        Uuid::new_v4(), // 随机用户ID
        "TEST_PILE".to_string(),
        ChargingMode::Fast,
        10.0, // 充电量
        0.5,  // 充电时长
        8.0,  // 充电费用
        2.0,  // 服务费用
        Utc::now() - chrono::Duration::hours(1), // 开始时间
        Utc::now(), // 结束时间
    );
    
    match test_record.insert(&pool).await {
        Ok(_) => {
            println!("✅ 测试充电详单插入成功");
            Ok(HttpResponse::Ok().json(json!({
                "success": true,
                "message": "测试充电详单插入成功",
                "record_id": test_record.id
            })))
        }
        Err(e) => {
            println!("❌ 测试充电详单插入失败: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": format!("测试插入失败: {}", e)
            })))
        }
    }
}

/// 配置充电详单路由
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/charging-records")
            .route("/user/{user_id}", web::get().to(get_user_charging_records))
            .route("/test-insert", web::post().to(test_insert_record))
    );
} 