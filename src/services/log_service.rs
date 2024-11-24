use crate::db::MongoRepo; // Fixes missing `MongoRepo`
use crate::models::log::LogPayload; // Fixes missing `LogPayload`
use crate::services::websocket_queue::{RetryQueueEntry, WebSocketQueue};
use crate::websocket::server::WebSocketServer; // Fixes unresolved `WebSocketServer`
use actix_web::web; // Fixes `use of undeclared crate or module 'web'` // Fixes unresolved `websocket_queue`

pub async fn process_log(
    log: LogPayload,
    data: web::Data<MongoRepo>,
    websocket_server: web::Data<WebSocketServer>,
    websocket_queue: web::Data<WebSocketQueue>,
) -> Result<(), String> {
    log::info!("Processing log: {:?}", log);

    let collection = data.db.collection::<LogPayload>("logs");
    let inserted_log = collection
        .insert_one(&log, None)
        .await
        .map_err(|e| e.to_string())?;
    let log_id = inserted_log
        .inserted_id
        .as_object_id()
        .ok_or("Failed to retrieve inserted log ID")?;

    log::info!("Log inserted with ID: {}", log_id);
    println!("Log inserted with ID: {}", log_id);

    // Attempt to deliver the log via WebSocket
    let org_id = log.organization_id.ok_or("Organization ID missing")?;
    let app_id = log.application_id.ok_or("Application ID missing")?;

    if !websocket_server
        .push_log_id(org_id.clone(), app_id.clone(), log_id.clone())
        .await
    {
        // // If no connection is available, add to the retry queue
        // let retry_entry = RetryQueueEntry {
        //     organization_id: org_id,
        //     application_id: app_id,
        //     log_id,
        // };
        // websocket_queue.enqueue(retry_entry).await;
    }

    Ok(())
}
