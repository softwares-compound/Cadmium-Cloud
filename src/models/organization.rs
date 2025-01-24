use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Organization {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub org_name: String,
    pub admin_email: String,
    pub admin_password: String,
    pub cd_id: String,
    pub cd_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateOrganization {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub org_name: String,
    pub admin_email: String,
    pub admin_password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cd_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cd_secret: Option<String>,

}
