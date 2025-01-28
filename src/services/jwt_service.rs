use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
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

pub fn validate_jwt(token: &str) -> Result<String, String> {
    let secret = env::var("JWT_SECRET").map_err(|_| "JWT_SECRET not set".to_string())?;
    let validation = Validation::default();
    let decoded = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )
    .map_err(|_| "Invalid token".to_string())?;
    Ok(decoded.claims.sub)
}
