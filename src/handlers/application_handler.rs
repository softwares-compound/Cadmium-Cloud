use crate::db::MongoRepo;
use crate::models::application::Application;
use actix_web::{web, HttpResponse, Responder};
use mongodb::bson::oid::ObjectId;

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
