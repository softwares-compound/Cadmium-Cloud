use crate::db::MongoRepo;
use crate::models::user::User;
use actix_web::{web, HttpResponse, Responder};
use mongodb::bson::doc;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

pub async fn signup(
    payload: web::Json<SignupRequest>,
    data: web::Data<MongoRepo>,
) -> impl Responder {
    let collection = data.db.collection::<User>("users");

    // Check if user already exists
    if collection
        .find_one(doc! { "email": &payload.email }, None)
        .await
        .unwrap_or(None)
        .is_some()
    {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "error": "User already exists" }));
    }

    // Hash password and create user
    let user = User {
        id: None,
        first_name: payload.first_name.clone(),
        last_name: payload.last_name.clone(),
        email: payload.email.clone(),
        password_hash: User::hash_password(&payload.password),
    };

    // Insert into MongoDB
    if collection.insert_one(user, None).await.is_err() {
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({ "error": "Signup failed" }));
    }

    HttpResponse::Ok().json(serde_json::json!({ "message": "Signup successful" }))
}
