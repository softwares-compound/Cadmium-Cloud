use crate::db::MongoRepo;
use crate::models::organization::Organization;
use actix_web::{web, HttpResponse, Responder,HttpRequest};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::doc;

pub async fn create_organization(
    payload: web::Json<Organization>,
    data: web::Data<MongoRepo>,
) -> impl Responder {
    let mut org = payload.into_inner();
    org.id = Some(ObjectId::new());
    match data.create_organization(org).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Organization created"})),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}


pub async fn get_organization_details(
    req: HttpRequest,
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

    // Find organization by CD-ID and CD-SECRET
    let collection = data.db.collection::<mongodb::bson::Document>("organizations");
    let filter = doc! { "cd_id": cd_id, "cd_secret": cd_secret };

    match collection.find_one(filter, None).await {
        Ok(Some(org)) => {
            let id = org.get_object_id("_id").ok();
            let org_name = org.get_str("org_name").ok();
            if let (Some(id), Some(org_name)) = (id, org_name) {
                HttpResponse::Ok().json(serde_json::json!({
                    "id": id.to_hex(),
                    "org_name": org_name,
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Incomplete organization data",
                }))
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid CD-ID or CD-SECRET",
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to fetch organization details",
        })),
    }
}
