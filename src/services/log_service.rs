use actix_web::web;  // Added missing import
use crate::models::log::LogPayload;
use crate::db::MongoRepo;
use log::info;


pub async fn process_log(log: LogPayload, data: web::Data<MongoRepo>) -> Result<(), String> {
    info!("Processing log: {:?}", log);
    let collection = data.db.collection::<LogPayload>("logs");
    collection.insert_one(log, None).await.map_err(|e| e.to_string())?;
    Ok(())
}