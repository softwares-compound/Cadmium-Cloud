use mongodb::{
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client, Database,
    bson::{doc, oid::ObjectId}, // Add this line to import ObjectId
};
use std::env;
use crate::models::{organization::Organization, application::Application};

#[derive(Clone)]
pub struct MongoRepo {
    pub db: Database,
}

impl MongoRepo {
    pub async fn init() -> Self {
        let uri = env::var("MONGODB_URI").expect("MONGODB_URI must be set");
        let mut client_options = ClientOptions::parse(&uri).await.expect("Failed to parse client options");

        // Set the server API version
        let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
        client_options.server_api = Some(server_api);

        let client = Client::with_options(client_options).expect("Failed to initialize client");
        let db_name = env::var("MONGODB_DB").expect("MONGODB_DB must be set");
        let db = client.database(&db_name);
        MongoRepo { db }
    }

    // Organization CRUD operations
    pub async fn create_organization(&self, org: Organization) -> Result<(), mongodb::error::Error> {
        let collection = self.db.collection::<Organization>("organizations");
        collection.insert_one(org, None).await?;
        Ok(())
    }

    pub async fn get_organization_by_id(&self, org_id: ObjectId) -> Result<Option<Organization>, mongodb::error::Error> {
        let collection = self.db.collection::<Organization>("organizations");
        let filter = doc! { "_id": org_id };
        collection.find_one(filter, None).await
    }

    // Application CRUD operations
    pub async fn create_application(&self, app: Application) -> Result<(), mongodb::error::Error> {
        let collection = self.db.collection::<Application>("applications");
        collection.insert_one(app, None).await?;
        Ok(())
    }

    pub async fn get_application_by_id(&self, app_id: ObjectId) -> Result<Option<Application>, mongodb::error::Error> {
        let collection = self.db.collection::<Application>("applications");
        let filter = doc! { "_id": app_id };
        collection.find_one(filter, None).await
    }
}
