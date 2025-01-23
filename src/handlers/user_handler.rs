// src/handlers/user_handler.rs
use crate::{
    db::MongoRepo,
    models::user::User,
    services::{jwt_service, otp_service},
};
use actix_web::cookie::Cookie;
use actix_web::{web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};
use std::env;
use std::fs;

pub async fn send_otp(payload: web::Json<String>) -> impl Responder {
    let email = payload.into_inner();
    let otp = otp_service::generate_otp(&email);
    // Read the HTML template
    // let template_path = Path::new("templates/otp_template.html");
    let base_path = env::current_dir().unwrap();
    let template_path = base_path.join("src/templates/otp_template.html");

    // println!("template_path: {:?}", template_path);
    // println!(
    //     "Current working directory: {:?}",
    //     std::env::current_dir().unwrap()
    // );

    let email_template = fs::read_to_string(template_path)
        .expect("Failed to read OTP email template")
        .replace("{{OTP_CODE}}", &otp);

    let body = serde_json::json!({
        "from": "noreply@neocadmium.softwarescompound.in",
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
    payload: web::Json<(String, String, String, Option<String>, String, String)>,
) -> impl Responder {
    let (email, otp, first_name, middle_name, last_name, password) = payload.into_inner();

    if !otp_service::verify_otp(&email, &otp) {
        return HttpResponse::BadRequest().json("Invalid OTP");
    }

    let password_hash = hash(password, DEFAULT_COST).unwrap();
    let user = User {
        id: None,
        first_name,
        middle_name,
        last_name,
        email: email.clone(),
        password_hash,
    };

    let collection = db.db.collection::<User>("users");
    let _ = collection.insert_one(user, None).await;

    let jwt = jwt_service::generate_jwt(&email);

    HttpResponse::Ok()
        .cookie(
            Cookie::build("auth_token", jwt)
                .http_only(true)
                .secure(true)
                .finish(),
        )
        .json("Signup successful")
}
