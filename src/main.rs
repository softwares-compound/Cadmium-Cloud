// File: ./src/main.rs
use actix_web::{App, HttpServer, middleware, web};
use dotenv::dotenv;

mod config;
mod logger;
mod routes;
mod db;
mod handlers;
mod services;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    logger::init();

    let mongo_repo = db::MongoRepo::init().await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mongo_repo.clone()))
            .wrap(middleware::Logger::default())
            .configure(routes::init)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
