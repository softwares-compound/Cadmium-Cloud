use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogPayload {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<ObjectId>,
    pub error: String,
    pub traceback: String,
    pub url: String,
    pub method: String,
}
