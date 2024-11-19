use mongodb::{options::{ClientOptions, ServerApi, ServerApiVersion}, Client, Database};
use std::env;

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
}
