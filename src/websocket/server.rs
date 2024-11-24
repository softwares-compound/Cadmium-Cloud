use crate::websocket::connection::WebSocketActor;
use actix::Addr;
use mongodb::bson::oid::ObjectId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents the global state of WebSocket connections.
#[derive(Clone)]
pub struct WebSocketServer {
    connections: Arc<RwLock<HashMap<String, HashMap<String, Vec<Addr<WebSocketActor>>>>>>,
}

impl WebSocketServer {
    /// Creates a new WebSocketServer instance.
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adds a WebSocket connection for a specific organization and application.
    pub async fn add_connection(
        &self,
        org_id: ObjectId,
        app_id: ObjectId,
        conn: Addr<WebSocketActor>,
    ) {
        let org_id_str = org_id.to_string();
        let app_id_str = app_id.to_string();

        let mut connections = self.connections.write().await;
        connections
            .entry(org_id_str)
            .or_default()
            .entry(app_id_str)
            .or_default()
            .push(conn);

        log::info!(
            "WebSocket connection added for Org ID: {}, App ID: {}",
            org_id,
            app_id
        );
    }

    /// Removes a WebSocket connection for a specific organization and application.
    pub async fn remove_connection(
        &self,
        org_id: ObjectId,
        app_id: ObjectId,
        conn: Addr<WebSocketActor>,
    ) {
        let org_id_str = org_id.to_string();
        let app_id_str = app_id.to_string();

        let mut connections = self.connections.write().await;
        if let Some(app_map) = connections.get_mut(&org_id_str) {
            if let Some(conn_list) = app_map.get_mut(&app_id_str) {
                conn_list.retain(|c| c != &conn);
                log::info!(
                    "WebSocket connection removed for Org ID: {}, App ID: {}",
                    org_id,
                    app_id
                );
            }
        }
    }

    /// Gets one WebSocket connection for a specific organization and application.
    pub async fn get_connection(
        &self,
        org_id: ObjectId,
        app_id: ObjectId,
    ) -> Option<Addr<WebSocketActor>> {
        let org_id_str = org_id.to_string();
        let app_id_str = app_id.to_string();

        let connections = self.connections.read().await;
        connections
            .get(&org_id_str)
            .and_then(|app_map| app_map.get(&app_id_str))
            .and_then(|conn_list| conn_list.get(0).cloned())
    }

    /// Pushes a log ID to one connection for a specific organization and application.
    pub async fn push_log_id(&self, org_id: ObjectId, app_id: ObjectId, log_id: ObjectId) -> bool {
        if let Some(conn) = self.get_connection(org_id, app_id).await {
            log::info!(
                "Pushing log ID: {} to WebSocket connection for Org ID: {}, App ID: {}",
                log_id,
                org_id,
                app_id
            );
            println!("Pushing log ID: {} to WebSocket connection for Org ID: {}, App ID: {}", log_id, org_id, app_id);
            conn.do_send(crate::websocket::connection::SendLogId { log_id });
            true
        } else {
            log::warn!(
                "No WebSocket connection found for Org ID: {}, App ID: {}",
                org_id,
                app_id
            );
            println!("No WebSocket connection found for Org ID: {}, App ID: {}", org_id, app_id);
            false
        }
    }
}
