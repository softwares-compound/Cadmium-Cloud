use crate::db::MongoRepo;
use crate::models::log::LogPayload;
use crate::services::log_service;
use crate::services::websocket_queue::WebSocketQueue;
use crate::websocket::server::WebSocketServer;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde_json::Value;
use log::{error, info};
use mongodb::bson;

use mongodb::bson::{doc, oid::ObjectId};
use mongodb::options::FindOptions;
use futures_util::stream::TryStreamExt; 


pub async fn save_log(
    req: HttpRequest,
    payload: web::Json<LogPayload>,
    data: web::Data<MongoRepo>,
    websocket_server: web::Data<WebSocketServer>,
    websocket_queue: web::Data<WebSocketQueue>,
) -> impl Responder {
    let cd_id = match req.headers().get("CD-ID") {
        Some(value) => match value.to_str() {
            Ok(v) => {
                log::debug!("Received CD-ID: {}", v);
                v
            }
            Err(_) => {
                log::error!("Invalid CD-ID header encoding");
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid CD-ID header encoding"
                }));
            }
        },
        None => {
            log::error!("Missing CD-ID header");
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Missing CD-ID header"
            }));
        }
    };

    let cd_secret = match req.headers().get("CD-Secret") {
        Some(value) => match value.to_str() {
            Ok(v) => {
                log::debug!("Received CD-Secret: {}", v);
                v
            }
            Err(_) => {
                log::error!("Invalid CD-Secret header encoding");
                return HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Invalid CD-Secret header encoding"
                }));
            }
        },
        None => {
            log::error!("Missing CD-Secret header");
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Missing CD-Secret header"
            }));
        }
    };

    let application_id = match req.headers().get("Application-ID") {
        Some(value) => match value.to_str() {
            Ok(v) => v,
            Err(_) => {
                error!("Invalid Application-ID header");
                return HttpResponse::BadRequest().body("Invalid Application-ID header");
            }
        },
        None => {
            error!("Missing Application-ID header");
            return HttpResponse::BadRequest().body("Missing Application-ID header");
        }
    };

    let org = match data
        .get_organization_by_cd_id_and_secret(cd_id, cd_secret)
        .await
    {
        Ok(Some(org)) => {
            log::info!("Successfully authenticated organization: {}", org.org_name);
            org
        }
        Ok(None) => {
            log::warn!("Authentication failed for CD-ID: {}", cd_id);
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid CD-ID or CD-Secret"
            }));
        }
        Err(e) => {
            log::error!("Database error during organization lookup: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Internal server error during authentication"
            }));
        }
    };

    let app_id = match ObjectId::parse_str(application_id) {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid Application-ID format");
            return HttpResponse::BadRequest().body("Invalid Application-ID format");
        }
    };

    let app = match data.get_application_by_id(app_id).await {
        Ok(Some(app)) => app,
        Ok(None) => {
            error!("Application not found");
            return HttpResponse::NotFound().body("Application not found");
        }
        Err(e) => {
            error!("Database error: {}", e);
            return HttpResponse::InternalServerError().body("Database error");
        }
    };

    // Ensure the application belongs to the organization
    if app.organization_id != Some(org.id.unwrap()) {
        error!("Application does not belong to the organization");
        return HttpResponse::Unauthorized()
            .body("Application does not belong to the organization");
    }
    // Process the log
    let mut log = payload.into_inner();
    log.organization_id = Some(org.id.unwrap());
    log.application_id = Some(app.id.unwrap());

    match log_service::process_log(
        log,
        data.clone(),
        websocket_server.clone(),
        websocket_queue.clone(),
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Log saved"})),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}




pub async fn get_log_by_id(
    req: HttpRequest,
    path: web::Path<String>,
    data: web::Data<MongoRepo>,
) -> impl Responder {
    // Extract headers for authentication
    let cd_id = req
        .headers()
        .get("CD-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let cd_secret = req
        .headers()
        .get("CD-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let app_id = req
        .headers()
        .get("Application-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    // Validate ObjectId from URL
    let log_id = match ObjectId::parse_str(path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid log ID format"
            }));
        }
    };

    // Authenticate organization using CD-ID and CD-Secret
    let org = match data.get_organization_by_cd_id_and_secret(cd_id, cd_secret).await {
        Ok(Some(org)) => org,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid CD-ID or CD-Secret"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to authenticate organization"
            }));
        }
    };

    // Validate Application-ID
    let app_id = match ObjectId::parse_str(app_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid Application-ID format"
            }));
        }
    };

    // Ensure application belongs to the authenticated organization
    let app = match data.get_application_by_id(app_id).await {
        Ok(Some(app)) if app.organization_id == Some(org.id.unwrap()) => app,
        Ok(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Application does not belong to the organization"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to validate application"
            }));
        }
    };

    // Fetch log from database
    let collection = data.db.collection::<LogPayload>("logs");
    match collection.find_one(doc! { "_id": log_id, "application_id": app.id.unwrap() }, None).await
    {
        Ok(Some(log)) => HttpResponse::Ok().json(log),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Log not found"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to retrieve log"
        })),
    }
}


/// Fetch all logs for a specific organization and application.
pub async fn get_all_logs(
    req: HttpRequest,
    data: web::Data<MongoRepo>,
) -> impl Responder {
    // Extract headers
    let cd_id = req
        .headers()
        .get("CD-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let cd_secret = req
        .headers()
        .get("CD-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let app_id = req
        .headers()
        .get("Application-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    // Authenticate organization using CD-ID and CD-Secret
    let org = match data.get_organization_by_cd_id_and_secret(cd_id, cd_secret).await {
        Ok(Some(org)) => org,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid CD-ID or CD-Secret"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to authenticate organization"
            }));
        }
    };

    // Validate Application-ID
    let app_id = match mongodb::bson::oid::ObjectId::parse_str(app_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid Application-ID format"
            }));
        }
    };

    let app = match data.get_application_by_id(app_id).await {
        Ok(Some(app)) if app.organization_id == Some(org.id.unwrap()) => app,
        Ok(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Application does not belong to the organization"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to validate application"
            }));
        }
    };

    // Fetch logs from the database
    let collection = data.db.collection::<LogPayload>("logs");
    let filter = doc! { "application_id": app.id.unwrap() };

    let find_options = FindOptions::builder()
        .sort(doc! { "_id": -1 }) // Optional: Sort by descending order of insertion
        .build();

    match collection.find(filter, find_options).await {
        Ok(mut cursor) => {
            let mut logs = Vec::new();
            while let Some(log) = cursor.try_next().await.unwrap_or(None) {
                logs.push(log);
            }
            HttpResponse::Ok().json(logs)
        }
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to retrieve logs"
        })),
    }
}


/// Update the `rag_inference` field of a specific log.
pub async fn update_rag_inference(
    req: HttpRequest,
    path: web::Path<String>,
    data: web::Data<MongoRepo>,
    payload: web::Json<Value>, // The `rag_inference` data
) -> impl Responder {
    // Extract headers for authentication
    let cd_id = req
        .headers()
        .get("CD-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let cd_secret = req
        .headers()
        .get("CD-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let app_id = req
        .headers()
        .get("Application-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    // Parse `log_id` from the URL
    let log_id = match ObjectId::parse_str(path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid log ID format"
            }));
        }
    };

    // Authenticate the organization using `CD-ID` and `CD-Secret`
    let org = match data.get_organization_by_cd_id_and_secret(cd_id, cd_secret).await {
        Ok(Some(org)) => org,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid CD-ID or CD-Secret"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to authenticate organization"
            }));
        }
    };

    // Validate the `Application-ID`
    let app_id = match ObjectId::parse_str(app_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid Application-ID format"
            }));
        }
    };

    let app = match data.get_application_by_id(app_id).await {
        Ok(Some(app)) if app.organization_id == Some(org.id.unwrap())=> app,
        Ok(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Application does not belong to the organization"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to validate application"
            }));
        }
    };

    // Update the `rag_inference` field in the log
    let collection = data.db.collection::<LogPayload>("logs");
    let filter = doc! {
        "_id": log_id,
        "application_id": app.id.unwrap(),
    };
    let update = doc! {
        "$set": {
            "rag_inference": bson::to_bson(&payload.into_inner()).unwrap_or(bson::Bson::Null),
        }
    };

    match collection.update_one(filter, update, None).await {
        Ok(update_result) if update_result.matched_count > 0 => {
            info!("Updated `rag_inference` for log ID: {}", log_id);
            HttpResponse::Ok().json(serde_json::json!({
                "message": "RAG inference updated successfully"
            }))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Log not found"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to update RAG inference"
        })),
    }
}