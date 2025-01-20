use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use mongodb::bson::oid::ObjectId;
use password_hash::{rand_core::OsRng, PasswordHash, SaltString};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password_hash: String, // Store hashed password
}

impl User {
    /// Hash a plain-text password before storing
    pub fn hash_password(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng); // Generate a random salt
        let argon2 = Argon2::default(); // Use default Argon2 config
        let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap();
        password_hash.to_string() // Convert to string format
    }

    /// Verify a password against the stored hash
    pub fn verify_password(&self, password: &str) -> bool {
        let parsed_hash = PasswordHash::new(&self.password_hash).unwrap();
        let argon2 = Argon2::default();
        argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }
}
