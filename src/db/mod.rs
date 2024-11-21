use crate::models::{application::Application, organization::Organization};
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client, Database,
};
use std::env;

#[derive(Clone)]
pub struct MongoRepo {
    pub db: Database,
}

impl MongoRepo {
    pub async fn init() -> Self {
        let uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
        let mut client_options = ClientOptions::parse(&uri)
            .await
            .expect("Failed to parse client options");

        // Set the server API version
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);

        let client = Client::with_options(client_options).expect("Failed to initialize client");
        let db_name = env::var("MONGODB_DB").expect("MONGODB_DB must be set");
        let db = client.database(&db_name);
        MongoRepo { db }
    }

    // Organization CRUD operations
    pub async fn create_organization(
        &self,
        org: Organization,
    ) -> Result<(), mongodb::error::Error> {
        let collection = self.db.collection::<Organization>("organizations");
        collection.insert_one(org, None).await?;
        Ok(())
    }
    // Application CRUD operations
    pub async fn create_application(&self, app: Application) -> Result<(), mongodb::error::Error> {
        let collection = self.db.collection::<Application>("applications");
        collection.insert_one(app, None).await?;
        Ok(())
    }
    pub async fn get_organization_by_cd_id_and_secret(
        &self,
        cd_id: &str,
        cd_secret: &str,
    ) -> Result<Option<Organization>, mongodb::error::Error> {
        let collection = self.db.collection::<Organization>("organizations");

        // Clean the incoming credentials
        let clean_cd_id = cd_id.trim().trim_matches('"');
        let clean_cd_secret = cd_secret.trim().trim_matches('"');
        println!("CD-ID: {}, CD-Secret: {}", clean_cd_id, clean_cd_secret);

        log::debug!(
            "Attempting to find organization with CD-ID: {} and CD-Secret: {}",
            clean_cd_id,
            clean_cd_secret
        );

        // Use a more flexible query with $regex for exact matching
        let filter = doc! {
            "cd_id": clean_cd_id.to_string(),
            "cd_secret": clean_cd_secret.to_string()
        };

        log::debug!("Query filter: {:?}", filter);
        println!("Query filter: {:?}", filter);
        let result = collection.find_one(filter, None).await?;

        if let Some(ref org) = result {
            log::info!("Found organization: {}", org.org_name);
            log::debug!(
                "Stored CD-ID: {}, CD-Secret: {}",
                org.cd_id.trim_matches('"'),
                org.cd_secret.trim_matches('"')
            );
        } else {
            log::warn!("No organization found for the provided credentials");
        }

        Ok(result)
    }

    pub async fn get_application_by_id(
        &self,
        app_id: ObjectId,
    ) -> Result<Option<Application>, mongodb::error::Error> {
        let collection = self.db.collection::<Application>("applications");
        let filter = doc! { "_id": app_id };

        match collection.find_one(filter, None).await {
            Ok(Some(app)) => {
                log::debug!("Found application with ID: {}", app_id);
                Ok(Some(app))
            }
            Ok(None) => {
                log::warn!("No application found for ID: {}", app_id);
                Ok(None)
            }
            Err(e) => {
                log::error!("Database error while looking up application: {}", e);
                Err(e)
            }
        }
    }
}
