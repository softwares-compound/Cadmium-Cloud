use crate::db::MongoRepo;
use crate::models::log::LogPayload;
use crate::models::application::Application; 
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use mongodb::bson::{doc, oid::ObjectId, Bson, DateTime};
use chrono::{NaiveDateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use futures_util::stream::TryStreamExt;

pub async fn create_application(
    req: HttpRequest,
    payload: web::Json<Application>,
    data: web::Data<MongoRepo>,
) -> impl Responder {
    // Extract CD-ID and CD-SECRET headers
    let cd_id = req
        .headers()
        .get("CD-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let cd_secret = req
        .headers()
        .get("CD-SECRET")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    // Authenticate organization using CD-ID and CD-SECRET
    let org = match data.get_organization_by_cd_id_and_secret(cd_id, cd_secret).await {
        Ok(Some(org)) => org,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid CD-ID or CD-SECRET"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to authenticate organization"
            }));
        }
    };

    // Prepare the application
    let mut app = payload.into_inner();
    let app_id = ObjectId::new(); // Generate a new ObjectId
    app.id = Some(app_id.clone());
    app.organization_id = Some(org.id.unwrap()); // Set the authenticated organization's ID

    // Save the application to the database
    match data.create_application(app).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Application created",
            "application_id": app_id.to_string() // Include the created application ID
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub async fn get_applications(
    req: HttpRequest,
    data: web::Data<MongoRepo>,
) -> impl Responder {
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

    let app_collection = data.db.collection::<Application>("applications");
    let log_collection = data.db.collection::<LogPayload>("logs");
    let filter = doc! { "organization_id": org.id.unwrap() };

    match app_collection.find(filter, None).await {
        Ok(mut cursor) => {
            let mut applications = Vec::new();

            while let Some(app) = cursor.try_next().await.unwrap_or(None) {
                let app_id = app.id.unwrap();
                let organization_id = app.organization_id.unwrap();

                let total_logs = log_collection
                    .count_documents(doc! { "application_id": app_id.clone() }, None)
                    .await
                    .unwrap_or(0);

                let resolved_logs = log_collection
                    .count_documents(
                        doc! {
                            "application_id": app_id.clone(),
                            "rag_inference": { "$ne": Bson::Null }
                        },
                        None,
                    )
                    .await
                    .unwrap_or(0);

                let today = Utc::now().naive_utc().date();
                let start_of_day = NaiveDateTime::new(today, chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap());
                let end_of_day = NaiveDateTime::new(today, chrono::NaiveTime::from_hms_opt(23, 59, 59).unwrap());

                let start_system_time = UNIX_EPOCH + Duration::from_millis(start_of_day.and_utc().timestamp_millis() as u64);
                let end_system_time = UNIX_EPOCH + Duration::from_millis(end_of_day.and_utc().timestamp_millis() as u64);

                let todays_logs = log_collection
                    .count_documents(
                        doc! {
                            "application_id": app_id.clone(),
                            "created_at": {
                                "$gte": DateTime::from_system_time(start_system_time),
                                "$lt": DateTime::from_system_time(end_system_time),
                            }
                        },
                        None,
                    )
                    .await
                    .unwrap_or(0);

                applications.push(serde_json::json!({
                    "_id": { "$oid": app_id.to_string() },
                    "organization_id": { "$oid": organization_id.to_string() },
                    "application_name": app.application_name,
                    "total_logs": total_logs,
                    "resolved_logs": resolved_logs,
                    "todays_logs": todays_logs
                }));
            }

            HttpResponse::Ok().json(applications)
        }
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to fetch applications"
        })),
    }
}

pub async fn delete_application(
    req: HttpRequest,
    path: web::Path<String>, // Extract application ID from the URL
    data: web::Data<MongoRepo>,
) -> impl Responder {
    // Extract CD-ID and CD-SECRET headers
    let cd_id = req
        .headers()
        .get("CD-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let cd_secret = req
        .headers()
        .get("CD-SECRET")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    // Authenticate organization using CD-ID and CD-SECRET
    let org = match data.get_organization_by_cd_id_and_secret(cd_id, cd_secret).await {
        Ok(Some(org)) => org,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid CD-ID or CD-SECRET"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to authenticate organization"
            }));
        }
    };

    // Validate and parse application_id from the URL
    let application_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid application ID format"
            }));
        }
    };

    // Ensure the application belongs to the authenticated organization
    let app = match data.get_application_by_id(application_id).await {
        Ok(Some(app)) if app.organization_id == Some(org.id.unwrap()) => app,
        Ok(_) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Application does not belong to the authenticated organization"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to validate application"
            }));
        }
    };

    // Delete the application
    let collection = data.db.collection::<Application>("applications");
    match collection.delete_one(doc! { "_id": app.id.unwrap() }, None).await {
        Ok(delete_result) if delete_result.deleted_count > 0 => {
            HttpResponse::Ok().json(serde_json::json!({
                "message": "Application deleted successfully"
            }))
        }
        Ok(_) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Application not found"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to delete application"
        })),
    }
}

pub async fn get_application(
    req: HttpRequest,
    path: web::Path<String>, // Extract application ID from the URL
    data: web::Data<MongoRepo>,
) -> impl Responder {
    // Extract CD-ID and CD-SECRET headers
    let cd_id = req
        .headers()
        .get("CD-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let cd_secret = req
        .headers()
        .get("CD-SECRET")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    // Authenticate organization using CD-ID and CD-SECRET
    let org = match data.get_organization_by_cd_id_and_secret(cd_id, cd_secret).await {
        Ok(Some(org)) => org,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid CD-ID or CD-SECRET"
            }));
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to authenticate organization"
            }));
        }
    };

    // Validate and parse application_id from the URL
    let application_id = match ObjectId::parse_str(&path.into_inner()) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid application ID format"
            }));
        }
    };

    // Fetch the application by ID
    let collection = data.db.collection::<Application>("applications");
    match collection
        .find_one(doc! { "_id": application_id, "organization_id": org.id.unwrap() }, None)
        .await
    {
        Ok(Some(application)) => HttpResponse::Ok().json(application),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Application not found"
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to fetch application"
        })),
    }
}
