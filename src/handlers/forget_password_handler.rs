use std::env;

use crate::{
    db::MongoRepo,
    models::user::User,
    services::{email_service::EmailService, otp_service},
};
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use mongodb::bson::doc;
use serde::Deserialize;
use std::fs;

/// Request payload for password reset
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub email: String,
}

/// Payload for verifying OTP
#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub email: String,
    pub otp: String,
}

/// Payload for setting a new password
#[derive(Debug, Deserialize)]
pub struct NewPasswordRequest {
    pub email: String,
    pub otp: String,
    pub new_password: String,
}

/// **Step 1: Send OTP for Password Reset**
pub async fn send_reset_otp(
    payload: web::Json<ResetPasswordRequest>,
    db: web::Data<MongoRepo>,
) -> impl Responder {
    let email = payload.into_inner().email;

    // Check if user exists
    let collection = db.db.collection::<User>("users");
    let user_exists = collection
        .find_one(doc! { "email": &email }, None)
        .await
        .unwrap();

    if user_exists.is_none() {
        return HttpResponse::NotFound().json("User with this email does not exist");
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

    let email_service = EmailService::new();
    match email_service
        .send_email(&email, "Password Reset OTP", &email_body)
        .await
    {
        Ok(_) => HttpResponse::Ok().json("OTP sent for password reset"),
        Err(e) => HttpResponse::InternalServerError().json(format!("Failed to send email: {}", e)),
    }
}

/// **Step 2: Verify OTP**
pub async fn verify_forgot_password_otp(
    payload: web::Json<VerifyOtpRequest>,
    db: web::Data<MongoRepo>,
) -> impl Responder {
    let payload = payload.into_inner();

    if !otp_service::verify_otp(&payload.email, &payload.otp, &db).await {
        return HttpResponse::BadRequest().json("Invalid or expired OTP");
    }

    HttpResponse::Ok().json("OTP verified successfully")
}

/// **Step 3: Reset Password**
pub async fn reset_password(
    payload: web::Json<NewPasswordRequest>,
    db: web::Data<MongoRepo>,
) -> impl Responder {
    let payload = payload.into_inner();

    // Verify OTP before allowing password change
    if !otp_service::verify_and_delete_otp(&payload.email, &payload.otp, &db).await {
        return HttpResponse::BadRequest().json("Invalid or expired OTP");
    }

    let password_hash = hash(&payload.new_password, DEFAULT_COST).unwrap();
    let collection = db.db.collection::<User>("users");

    let update_result = collection
        .update_one(
            doc! { "email": &payload.email },
            doc! { "$set": { "password_hash": password_hash } },
            None,
        )
        .await
        .unwrap();

    if update_result.matched_count == 0 {
        return HttpResponse::NotFound().json("User not found");
    }

    HttpResponse::Ok().json("Password updated successfully")
}
