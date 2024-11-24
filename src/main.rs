use actix_web::{middleware, web, App, HttpServer};

use dotenv::dotenv;
use tokio::task;
mod db;
mod handlers;
mod logger;
mod models;
mod routes;
mod services;
mod websocket;

use crate::websocket::server::WebSocketServer;
use crate::services::websocket_queue::WebSocketQueue;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    logger::init();

    let mongo_repo = db::MongoRepo::init().await;

    // Initialize the WebSocket server and queue
    let websocket_server = WebSocketServer::new();
    let websocket_queue = WebSocketQueue::new();

    let websocket_server_data = web::Data::new(websocket_server.clone());
    let websocket_queue_data = web::Data::new(websocket_queue.clone());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mongo_repo.clone()))
            .app_data(websocket_server_data.clone())
            .app_data(websocket_queue_data.clone())
            .wrap(middleware::Logger::default())
            .configure(routes::init)
    })
    .bind(("0.0.0.0", 8080))?;



    let server_result = server.run().await;

    // Optionally handle the queue processor task if needed
    // queue_processor.await.expect("Queue processor task failed");

    server_result
}