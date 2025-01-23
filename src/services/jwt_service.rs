// src/services/jwt_service.rs
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn generate_jwt(email: &str) -> String {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let expiration = Utc::now() + Duration::hours(24);
    let claims = Claims {
        sub: email.to_string(),
        exp: expiration.timestamp() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap()
}
