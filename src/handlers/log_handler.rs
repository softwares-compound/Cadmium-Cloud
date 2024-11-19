// src/handlers/log_handler.rs
use actix_web::{web, HttpResponse, Responder};
use crate::services::log_service;
use crate::models::log::LogPayload;
use crate::db::MongoRepo;  // Added missing import

pub async fn save_log(
    payload: web::Json<LogPayload>,
    data: web::Data<MongoRepo>,
) -> impl Responder {
    let log = payload.into_inner();
    match log_service::process_log(log, data).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Log saved"})),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}