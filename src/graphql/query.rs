use async_graphql::{Context, Object, Result, Error};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::options::FindOptions;
use futures_util::stream::TryStreamExt;
use crate::db::MongoRepo;
use crate::models::log::{LogPayload, LogPayloadGql};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Fetches logs for a specific application based on headers and pagination parameters.
    async fn logs(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<LogPayloadGql>> {
        let headers = ctx.data::<std::collections::HashMap<String, String>>()?;
        let cd_id = headers.get("CD-ID").ok_or_else(|| Error::new("Missing CD-ID"))?;
        let cd_secret = headers
            .get("CD-Secret")
            .ok_or_else(|| Error::new("Missing CD-Secret"))?;
        let app_id = headers
            .get("Application-ID")
            .ok_or_else(|| Error::new("Missing Application-ID"))?;

        let mongo_repo = ctx.data::<MongoRepo>()?;

        // Authenticate organization
        let org = mongo_repo
            .get_organization_by_cd_id_and_secret(cd_id, cd_secret)
            .await?
            .ok_or_else(|| Error::new("Invalid CD-ID or CD-Secret"))?;

        // Validate application
        let app_id = ObjectId::parse_str(app_id)
            .map_err(|_| Error::new("Invalid Application-ID format"))?;
        let app = mongo_repo
            .get_application_by_id(app_id)
            .await?
            .ok_or_else(|| Error::new("Application not found"))?;

        if app.organization_id != Some(org.id.unwrap()) {
            return Err(Error::new("Unauthorized"));
        }

        // Pagination logic
        let page = page.unwrap_or(1);
        let limit = limit.unwrap_or(10);
        let skip = (page - 1) * limit;

        // Fetch logs
        let collection = mongo_repo.db.collection::<LogPayload>("logs");
        let filter = doc! { "application_id": app.id.unwrap() };
        let find_options = FindOptions::builder()
            .skip(Some(skip as u64))
            .limit(Some(limit as i64))
            .sort(doc! { "created_at": -1 })
            .build();

        let mut cursor = collection.find(filter, find_options).await?;
        let mut logs = Vec::new();
        while let Some(log) = cursor.try_next().await? {
            logs.push(LogPayloadGql::from(log));
        }

        Ok(logs)
    }

    /// Fetches a single log by its ID.
    async fn log_by_id(
        &self,
        ctx: &Context<'_>,
        log_id: String,
    ) -> Result<LogPayloadGql> {
        let headers = ctx.data::<std::collections::HashMap<String, String>>()?;
        let cd_id = headers.get("CD-ID").ok_or_else(|| Error::new("Missing CD-ID"))?;
        let cd_secret = headers
            .get("CD-Secret")
            .ok_or_else(|| Error::new("Missing CD-Secret"))?;
        let app_id = headers
            .get("Application-ID")
            .ok_or_else(|| Error::new("Missing Application-ID"))?;

        let mongo_repo = ctx.data::<MongoRepo>()?;

        // Authenticate organization
        let org = mongo_repo
            .get_organization_by_cd_id_and_secret(cd_id, cd_secret)
            .await?
            .ok_or_else(|| Error::new("Invalid CD-ID or CD-Secret"))?;

        // Validate application
        let app_id = ObjectId::parse_str(app_id)
            .map_err(|_| Error::new("Invalid Application-ID format"))?;
        let app = mongo_repo
            .get_application_by_id(app_id)
            .await?
            .ok_or_else(|| Error::new("Application not found"))?;

        if app.organization_id != Some(org.id.unwrap()) {
            return Err(Error::new("Unauthorized"));
        }

        // Fetch the log
        let log_id = ObjectId::parse_str(&log_id)
            .map_err(|_| Error::new("Invalid Log ID format"))?;
        let collection = mongo_repo.db.collection::<LogPayload>("logs");
        let filter = doc! { "_id": log_id, "application_id": app.id.unwrap() };

        let log = collection
            .find_one(filter, None)
            .await?
            .ok_or_else(|| Error::new("Log not found"))?;

        Ok(LogPayloadGql::from(log))
    }
}
