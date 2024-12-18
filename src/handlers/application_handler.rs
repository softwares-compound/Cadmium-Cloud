use crate::db::MongoRepo;
use crate::models::application::{Application, DeleteApplicationPayload};
use actix_web::{web, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;
use actix_web::{ HttpRequest};
use mongodb::bson::doc;
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
    // Extract CD-ID and CD-Secret headers
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

    // Fetch all applications for the authenticated organization
    let collection = data.db.collection::<Application>("applications");
    let filter = doc! { "organization_id": org.id.unwrap() };

    match collection.find(filter, None).await {
        Ok(mut cursor) => {
            let mut applications = Vec::new();
            while let Some(app) = cursor.try_next().await.unwrap_or(None) {
                applications.push(app);
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
