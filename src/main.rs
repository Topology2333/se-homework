mod db;
mod models;
mod routes;

use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use actix_cors::Cors;
use routes::user::user_routes;
use routes::pile_api::pile_routes;
use routes::charging_request_api::charging_request_routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_pool = db::create_pool().await;

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .app_data(web::Data::new(db_pool.clone()))
            .configure(user_routes)
            .configure(pile_routes)
            .configure(charging_request_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
