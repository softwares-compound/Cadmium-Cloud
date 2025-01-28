use crate::services::email_service::EmailService;
use crate::{
    db::MongoRepo,
    models::user::User,
    services::{jwt_service, otp_service},
};
use actix_web::cookie::{time, Cookie};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use mongodb::bson::doc;
use serde::Deserialize;
use std::env;
use std::fs;
#[derive(Debug, Deserialize)]
pub struct SignupPayload {
    pub email: String,
    pub otp: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct OtpRequest {
    pub email: String,
}

pub async fn verify_email(
    payload: web::Json<OtpRequest>,
    db: web::Data<MongoRepo>,
) -> impl Responder {
    let email = payload.into_inner().email; // Extract email from JSON

    // Check if email is already registered
    let collection = db.db.collection::<User>("users");
    let existing_user = collection
        .find_one(doc! { "email": &email }, None)
        .await
        .unwrap();

    if existing_user.is_some() {
        return HttpResponse::Conflict()
            .json(serde_json::json!({ "message": "Email already registered" }));
    }

    let otp = otp_service::generate_otp(&email, &db).await;
    // Read the HTML template
    // let template_path = Path::new("templates/otp_template.html");
    let base_path = env::current_dir().unwrap();
    let template_path = base_path.join("src/templates/otp_template.html");
    let email_template = fs::read_to_string(template_path)
        .expect("Failed to read OTP email template")
        .replace("{{OTP_CODE}}", &otp);
    let email_body = email_template;

    // Send OTP using Resend API (use actix-web client)
    let email_service = EmailService::new();
    match email_service
        .send_email(&email, "Your OTP Code", &email_body)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "message": "OTP sent successfully" })),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": format!("Failed to send email: {}", e) })),
    }
}

pub async fn verify_and_delete_otp_and_signup(
    db: web::Data<MongoRepo>,
    payload: web::Json<SignupPayload>, // Change from tuple to struct
) -> impl Responder {
    let payload = payload.into_inner(); // Convert JSON to Rust struct

    // Check if email already exists BEFORE inserting
    let collection = db.db.collection::<User>("users");
    let existing_user = collection
        .find_one(doc! { "email": &payload.email }, None)
        .await
        .unwrap();

    if existing_user.is_some() {
        return HttpResponse::Conflict()
            .json(serde_json::json!({ "message": "Email already registered" }));
    }

    if !otp_service::verify_and_delete_otp(&payload.email, &payload.otp, &db).await {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({ "message": "Invalid or expired OTP" }));
    }

    let password_hash = hash(payload.password, DEFAULT_COST).unwrap();
    let user = User {
        id: None,
        first_name: payload.first_name,
        middle_name: payload.middle_name,
        last_name: payload.last_name,
        email: payload.email.clone(),
        password_hash,
    };

    let collection = db.db.collection::<User>("users");
    let _ = collection.insert_one(user, None).await;

    let jwt = jwt_service::generate_jwt(&payload.email);

    HttpResponse::Ok()
        .cookie(
            Cookie::build("auth_token", jwt)
                .http_only(true)
                .secure(true)
                .finish(),
        )
        .json(serde_json::json!({ "message": "Signup successful" }))
}

pub async fn logout() -> impl Responder {
    // Clear the auth_token cookie by setting an expired one
    // Sends a new cookie with an empty value.
    // Sets max_age = -1 to tell the browser to delete the cookie.
    // Uses path("/") to ensure it clears across the entire site.
    HttpResponse::Ok()
        .cookie(
            Cookie::build("auth_token", "")
                .http_only(true)
                .secure(true)
                .path("/") // Ensure it covers the entire domain
                .max_age(time::Duration::seconds(-1)) // Expire immediately
                .finish(),
        )
        .json(serde_json::json!({ "message": "Logout successful" }))
}

pub async fn validate_user(req: HttpRequest, db: web::Data<MongoRepo>) -> impl Responder {
    if let Some(cookie) = req.cookie("auth_token") {
        let token = cookie.value();
        let user_email = match jwt_service::validate_jwt(token) {
            Ok(email) => email,
            Err(_) => {
                return HttpResponse::Unauthorized()
                    .cookie(
                        Cookie::build("auth_token", "")
                            .max_age(time::Duration::seconds(-1))
                            .finish(),
                    )
                    .json(serde_json::json!({ "message": "Invalid token" }));
            }
        };

        let collection = db.db.collection::<User>("users");
        let user = collection
            .find_one(doc! { "email": &user_email }, None)
            .await
            .unwrap();

        if let Some(user) = user {
            return HttpResponse::Ok().json(serde_json::json!({ "data": user, "is_valid": true }));
        }
    }

    HttpResponse::Unauthorized()
        .cookie(
            Cookie::build("auth_token", "")
                .max_age(time::Duration::seconds(-1))
                .finish(),
        )
        .json(serde_json::json!({ "message": "Unauthorized" }))
}
