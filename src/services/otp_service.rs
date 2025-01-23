// src/services/otp_service.rs
use rand::Rng;
use std::collections::HashMap;
use std::sync::RwLock;

lazy_static::lazy_static! {
    static ref OTP_STORAGE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

pub fn generate_otp(email: &str) -> String {
    let otp: String = rand::thread_rng().gen_range(100000..999999).to_string();
    OTP_STORAGE
        .write()
        .unwrap()
        .insert(email.to_string(), otp.clone());
    otp
}

pub fn verify_otp(email: &str, otp: &str) -> bool {
    OTP_STORAGE
        .read()
        .unwrap()
        .get(email)
        .map_or(false, |stored| stored == otp)
}
