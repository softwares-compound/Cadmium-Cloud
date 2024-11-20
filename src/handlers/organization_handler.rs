use actix_web::{web, HttpResponse, Responder};
use crate::models::organization::Organization;
use crate::db::MongoRepo;
use mongodb::bson::oid::ObjectId;

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