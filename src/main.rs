mod db;
mod models;
mod routes;

use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use actix_cors::Cors;
use routes::user::user_routes;
use routes::pile_api::pile_routes;
use routes::charging_request_api::charging_request_routes;
use env_logger;
use routes::scheduler_api;
use routes::billing_api;
use routes::charging_record_api;
use charging_station::scheduler::init_global_scheduler_with_db;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    // 创建数据库连接池
    let db_pool = db::create_pool().await;

    // 初始化调度器并设置数据库连接池
    let scheduler = init_global_scheduler_with_db(Arc::new(db_pool.clone()));
    scheduler.start().await.expect("Failed to start scheduler");

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header())
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(scheduler.clone()))
            .configure(user_routes)
            .configure(pile_routes)
            .service(
                web::scope("/api")
                    .configure(charging_request_routes)
                    .configure(scheduler_api::config)
                    .configure(billing_api::config)
                    .configure(charging_record_api::config)
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
