use async_graphql::*;
use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(SimpleObject)]
pub struct LogPayloadGql {
    pub id: Option<String>,
    pub organization_id: Option<String>,
    pub application_id: Option<String>,
    pub error: String,
    pub traceback: String,
    pub url: String,
    pub method: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl From<LogPayload> for LogPayloadGql {
    fn from(log: LogPayload) -> Self {
        Self {
            id: log.id.map(|id| id.to_string()),
            organization_id: log.organization_id.map(|id| id.to_string()),
            application_id: log.application_id.map(|id| id.to_string()),
            error: log.error,
            traceback: log.traceback,
            url: log.url,
            method: log.method,
            created_at: log.created_at.map(|dt| dt.to_rfc3339()),
            updated_at: log.updated_at.map(|dt| dt.to_rfc3339()),
        }
    }
}