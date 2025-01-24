use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OtpEntry {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub email: String,
    pub otp: String,
    pub created_at: chrono::DateTime<Utc>, // Used for TTL index
}
