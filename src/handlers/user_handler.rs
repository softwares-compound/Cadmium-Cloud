use crate::{
    db::MongoRepo,
    models::user::User,
    services::{jwt_service, otp_service},
};
use actix_web::cookie::Cookie;
use actix_web::{web, HttpResponse, Responder};
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

pub async fn send_otp(payload: web::Json<OtpRequest>, db: web::Data<MongoRepo>) -> impl Responder {
    let email = payload.into_inner().email; // Extract email from JSON

    // Check if email is already registered
    let collection = db.db.collection::<User>("users");
    let existing_user = collection
        .find_one(doc! { "email": &email }, None)
        .await
        .unwrap();

    if existing_user.is_some() {
        return HttpResponse::Conflict().json("Email already registered");
    }

    let otp = otp_service::generate_otp(&email, &db).await;
    // Read the HTML template
    // let template_path = Path::new("templates/otp_template.html");
    let base_path = env::current_dir().unwrap();
    let template_path = base_path.join("src/templates/otp_template.html");

    let email_template = fs::read_to_string(template_path)
        .expect("Failed to read OTP email template")
        .replace("{{OTP_CODE}}", &otp);

    let body = serde_json::json!({
        "from": "Verification@neocadmium.softwarescompound.in",
        "to": email,
        "subject": "Your OTP Code",
        "html": email_template
    });

    // Send OTP using Resend API (use actix-web client)
    let client = reqwest::Client::new();
    let _ = client
        .post("https://api.resend.com/emails")
        .header(
            "Authorization",
            format!(
                " Bearer {}",
                env::var("RESEND_TOKEN").expect("RESEND_TOKEN must be set")
            ),
        )
        .json(&body)
        .send()
        .await;

    HttpResponse::Ok().json("OTP sent successfully")
}

pub async fn verify_otp_and_signup(
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
        return HttpResponse::Conflict().json("Email already registered");
    }

    if !otp_service::verify_otp(&payload.email, &payload.otp, &db).await {
        return HttpResponse::BadRequest().json("Invalid or expired OTP");
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
        .json("Signup successful")
}
