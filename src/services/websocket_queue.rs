use actix::Addr;
use log::{info, warn};
use mongodb::bson::oid::ObjectId;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents a log delivery retry queue entry.
#[derive(Debug, Clone)]
pub struct RetryQueueEntry {
    pub organization_id: ObjectId,
    pub application_id: ObjectId,
    pub log_id: ObjectId,
}

/// A thread-safe retry queue for undelivered logs.
#[derive(Clone)]
pub struct WebSocketQueue {
    queue: Arc<RwLock<VecDeque<RetryQueueEntry>>>,
}

impl WebSocketQueue {
    /// Creates a new WebSocketQueue instance.
    pub fn new() -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Adds a new log entry to the retry queue.
    pub async fn enqueue(&self, entry: RetryQueueEntry) {
        let mut queue = self.queue.write().await;
        queue.push_back(entry);
        info!("Log added to retry queue. Queue size: {}", queue.len());
        println!("Log added to retry queue. Queue size: {}", queue.len());
    }

    /// Retrieves and removes the oldest log entry from the retry queue, if available.
    pub async fn dequeue(&self) -> Option<RetryQueueEntry> {
        let mut queue = self.queue.write().await;
        let entry = queue.pop_front();
        if let Some(ref e) = entry {
            info!(
                "Dequeued log for Org ID: {}, App ID: {}, Log ID: {}",
                e.organization_id, e.application_id, e.log_id
            );
            println!(
                "Dequeued log for Org ID: {}, App ID: {}, Log ID: {}",
                e.organization_id, e.application_id, e.log_id
            );
        }
        entry
    }

    /// Returns the current size of the retry queue.
    pub async fn size(&self) -> usize {
        let queue = self.queue.read().await;
        queue.len()
    }

    /// Attempts to process the retry queue, delivering logs to active WebSocket connections.
    /// If delivery fails, the log is re-added to the queue for another retry.
    pub async fn process_queue<F>(&self, get_connection: F)
    where
        F: Fn(ObjectId, ObjectId) -> Option<Addr<crate::websocket::connection::WebSocketActor>>,
    {
        loop {
            let entry = self.dequeue().await;

            if let Some(log_entry) = entry {
                if let Some(conn) =
                    get_connection(log_entry.organization_id, log_entry.application_id)
                {
                    info!(
                        "Attempting to deliver log ID: {} to Org ID: {}, App ID: {}",
                        log_entry.log_id, log_entry.organization_id, log_entry.application_id
                    );
                    println!("Attempting to deliver log ID: {} to Org ID: {}, App ID: {}", log_entry.log_id, log_entry.organization_id, log_entry.application_id);

                    conn.do_send(crate::websocket::connection::SendLogId {
                        log_id: log_entry.log_id,app_id:log_entry.application_id
                    });
                } else {
                    warn!(
                        "No WebSocket connection found for Org ID: {}, App ID: {}. Re-queuing log ID: {}.",
                        log_entry.organization_id, log_entry.application_id, log_entry.log_id
                    );
                    println!("No WebSocket connection found for Org ID: {}, App ID: {}. Re-queuing log ID: {}.", log_entry.organization_id, log_entry.application_id, log_entry.log_id);

                    self.enqueue(log_entry).await;
                }
            }

            // Optional: Adjust this delay based on your requirements.
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }
}
