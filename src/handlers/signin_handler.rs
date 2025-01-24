use crate::{db::MongoRepo, models::user::User, services::jwt_service};
use actix_web::{cookie::Cookie, web, HttpResponse, Responder};
use bcrypt::verify;
use mongodb::bson::doc;
use serde::Serialize;

/// Payload for sign-in request
#[derive(Debug, serde::Deserialize)]
pub struct SigninRequest {
    pub email: String,
    pub password: String,
}

/// Response structure for successful sign-in
#[derive(Debug, Serialize)]
pub struct SigninResponse {
    pub message: String,
    pub data: UserResponse,
}

/// Response structure for returning user details (excluding password hash)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Option<String>,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub email: String,
}

/// **Sign-in API: Validates user credentials and returns user details with JWT**
pub async fn signin(payload: web::Json<SigninRequest>, db: web::Data<MongoRepo>) -> impl Responder {
    let payload = payload.into_inner();

    // Fetch user from database
    let collection = db.db.collection::<User>("users");
    let user = collection
        .find_one(doc! { "email": &payload.email }, None)
        .await
        .unwrap();

    if let Some(user) = user {
        // Verify the password
        if verify(payload.password, &user.password_hash).unwrap_or(false) {
            let jwt = jwt_service::generate_jwt(&user.email);

            // Convert user model to response format (excluding password hash)
            let user_response = UserResponse {
                id: user.id.map(|id| id.to_string()),
                first_name: user.first_name,
                middle_name: user.middle_name,
                last_name: user.last_name,
                email: user.email,
            };

            return HttpResponse::Ok()
                .cookie(
                    Cookie::build("auth_token", jwt) // Store JWT in HttpOnly Cookie
                        .http_only(true)
                        .secure(true)
                        .finish(),
                )
                .json(SigninResponse {
                    message: "Sign-in successful".to_string(),
                    data: user_response,
                });
        }
    }

    HttpResponse::Unauthorized().json("Invalid email or password")
}
