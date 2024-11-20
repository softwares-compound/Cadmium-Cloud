use actix_web::{web, HttpRequest, HttpResponse, Responder};
use crate::services::log_service;
use crate::models::log::LogPayload;
use crate::db::MongoRepo;
use mongodb::bson::oid::ObjectId;
use log::error;

pub async fn save_log(
    req: HttpRequest,
    payload: web::Json<LogPayload>,
    data: web::Data<MongoRepo>,
) -> impl Responder {
    let cd_id = match req.headers().get("CD-ID") {
        Some(value) => match value.to_str() {
            Ok(v) => {
                log::debug!("Received CD-ID: {}", v);
                v
            },
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
            },
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

    let org = match data.get_organization_by_cd_id_and_secret(cd_id, cd_secret).await {
        Ok(Some(org)) => {
            log::info!("Successfully authenticated organization: {}", org.org_name);
            org
        },
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
    if app.organization_id != org.id.unwrap() {
        error!("Application does not belong to the organization");
        return HttpResponse::Unauthorized().body("Application does not belong to the organization");
    }

    // Process the log
    let mut log = payload.into_inner();
    log.organization_id = Some(org.id.unwrap());
    log.application_id = Some(app.id.unwrap());

    match log_service::process_log(log, data).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Log saved"})),
        Err(e) => {
            error!("Failed to process log: {}", e);
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}
