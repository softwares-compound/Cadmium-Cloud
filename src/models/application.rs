use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Application {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(skip_serializing_if = "Option::is_none")] // Allows omission during deserialization
    pub organization_id: Option<ObjectId>,
    pub application_name: String,
}

#[derive(Deserialize)]
pub struct DeleteApplicationPayload {
    pub application_id: String,
}
