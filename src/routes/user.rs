use actix_web::{get, post, web, HttpResponse, Responder};
use crate::models::user::User;
use crate::models::ChargingRecord;
use sqlx::MySqlPool;
use serde_json::json;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub is_admin: bool,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[get("/users")]
async fn get_users(db_pool: web::Data<MySqlPool>) -> impl Responder {
    match User::get_all(db_pool.get_ref()).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/register")]
async fn register_user(
    db_pool: web::Data<MySqlPool>,
    new_user: web::Json<NewUser>,
) -> impl Responder {
    let new_user = new_user.into_inner();

    let mut hasher = Sha256::new();
    hasher.update(new_user.password);
    let password_hash = format!("{:x}", hasher.finalize());

    let user = User::new(new_user.username, password_hash, new_user.is_admin);
    match user.insert(db_pool.get_ref()).await {
        Ok(_) => HttpResponse::Created().body("注册成功"),
        Err(_) => HttpResponse::InternalServerError().body("注册失败"),
    }
}

#[post("/login")]
async fn login_user(
    db_pool: web::Data<MySqlPool>,
    login: web::Json<LoginRequest>,
) -> impl Responder {
    let login = login.into_inner();

    let mut hasher = Sha256::new();
    hasher.update(login.password);
    let password_hash = format!("{:x}", hasher.finalize());

    let result = sqlx::query_as!(
        User,
        r#"
        SELECT 
            id as "id: Uuid", 
            username, 
            password_hash, 
            is_admin as "is_admin: bool", 
            created_at as "created_at: chrono::DateTime<chrono::Utc>"
        FROM users
        WHERE username = ? AND password_hash = ?
        "#,
        login.username,
        password_hash
    )
    .fetch_optional(db_pool.get_ref())
    .await;

    match result {
        Ok(Some(_user)) => HttpResponse::Ok().json(_user),
        Ok(None) => HttpResponse::Unauthorized().body("用户名或密码错误"),
        Err(e) => {
            eprintln!("登录失败: {:?}", e);
            HttpResponse::InternalServerError().body("服务器异常")
        }
    }
}

#[get("/users/{user_id}/charging_records")] // 根据用户ID获取充电详单
async fn get_user_charging_records(
    db_pool: web::Data<MySqlPool>,
    path: web::Path<Uuid>, // 从路径中获取 user_id
) -> impl Responder {
    let user_id = path.into_inner();
    match ChargingRecord::find_by_user_id(user_id, db_pool.get_ref()).await { // 调用 ChargingRecord 的查询方法
        Ok(records) => HttpResponse::Ok().json(records),
        Err(e) => {
            eprintln!("Error fetching charging records: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to fetch charging records")
        },
    }
}

pub fn user_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_users)
       .service(register_user)
       .service(login_user)
       .service(get_user_charging_records); 
}
