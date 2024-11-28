use crate::db::MongoRepo;
use crate::models::application::Application;
use actix_web::{web, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;
use actix_web::{ HttpRequest};
use mongodb::bson::doc;
use futures_util::stream::TryStreamExt;


pub async fn create_application(
    payload: web::Json<Application>,
    data: web::Data<MongoRepo>,
) -> impl Responder {
    let mut app = payload.into_inner();
    app.id = Some(ObjectId::new());
    match data.create_application(app).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Application created"})),
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
