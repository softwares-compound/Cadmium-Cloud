use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpRequest, HttpServer};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use dotenv::dotenv; // Import Cors middleware
mod db;
mod graphql;
mod handlers;
mod logger;
mod middlewares;
mod models;
mod routes;
mod services;
mod websocket; // Make sure this module is declared

use crate::graphql::schema::{create_schema, AppSchema};
use crate::services::websocket_queue::WebSocketQueue;
use crate::websocket::server::WebSocketServer;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    logger::init();

    let mongo_repo = db::MongoRepo::init().await;
    // Ensure OTP TTL index is created
    // Call `setup_otp_ttl_index` as an associated function
    db::MongoRepo::setup_otp_ttl_index(&mongo_repo.db).await;

    // Initialize the WebSocket server and queue
    let websocket_server = WebSocketServer::new();
    let websocket_queue = WebSocketQueue::new();

    let websocket_server_data = web::Data::new(websocket_server.clone());
    let websocket_queue_data = web::Data::new(websocket_queue.clone());

    // Create GraphQL schema
    let schema = create_schema(mongo_repo.clone());
    let schema_data = web::Data::new(schema);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mongo_repo.clone()))
            .app_data(schema_data.clone())
            .app_data(websocket_server_data.clone())
            .app_data(websocket_queue_data.clone())
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default() // Configure CORS to allow all origins
                    .allowed_origin("http://localhost:6967") // ✅ Allow frontend origin
                    // .allow_any_origin() // Allow any origin (bypass CORS)
                    .allow_any_method() // Allow any HTTP method
                    .allow_any_header() // Allow any header
                    .supports_credentials(), // Explicitly allow credentials
            )
            .configure(routes::init)
            .route("/graphql", web::post().to(graphql_handler))
    })
    .bind(("0.0.0.0", 8080))?;

    let server_result = server.run().await;

    // Optionally handle the queue processor task if needed
    // queue_processor.await.expect("Queue processor task failed");

    server_result
}

async fn graphql_handler(
    schema: web::Data<AppSchema>,
    req: GraphQLRequest,
    http_req: HttpRequest,
) -> GraphQLResponse {
    let mut headers_map = std::collections::HashMap::new();

    // Extract headers
    if let Some(cd_id) = http_req
        .headers()
        .get("CD-ID")
        .and_then(|v| v.to_str().ok())
    {
        headers_map.insert("CD-ID".to_string(), cd_id.to_string());
    }
    if let Some(cd_secret) = http_req
        .headers()
        .get("CD-Secret")
        .and_then(|v| v.to_str().ok())
    {
        headers_map.insert("CD-Secret".to_string(), cd_secret.to_string());
    }
    if let Some(app_id) = http_req
        .headers()
        .get("Application-ID")
        .and_then(|v| v.to_str().ok())
    {
        headers_map.insert("Application-ID".to_string(), app_id.to_string());
    }

    let mut request = req.into_inner();
    request = request.data(headers_map);

    schema.execute(request).await.into()
}
