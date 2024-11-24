use actix_web::{HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use crate::websocket::connection::WebSocketActor;
use crate::websocket::server::WebSocketServer;
use crate::db::MongoRepo;

pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<MongoRepo>,
    websocket_server: web::Data<WebSocketServer>,
) -> HttpResponse {
    let cd_id = req.headers().get("CD-ID").and_then(|h| h.to_str().ok());
    let cd_secret = req.headers().get("CD-Secret").and_then(|h| h.to_str().ok());
    let app_id = req.headers().get("Application-ID").and_then(|h| h.to_str().ok());

    if cd_id.is_none() || cd_secret.is_none() || app_id.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Missing CD-ID, CD-Secret, or Application-ID headers"
        }));
    }

    let cd_id = cd_id.unwrap();
    let cd_secret = cd_secret.unwrap();
    let app_id = app_id.unwrap();

    let org = match data.get_organization_by_cd_id_and_secret(cd_id, cd_secret).await {
        Ok(Some(org)) => org,
        Ok(None) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid CD-ID or CD-Secret"
            }));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to authenticate organization"
            }));
        }
    };

    let app_id_parsed = match mongodb::bson::oid::ObjectId::parse_str(app_id) {
        Ok(id) => id,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid Application-ID format"
            }));
        }
    };

    let app = match data.get_application_by_id(app_id_parsed).await {
        Ok(Some(app)) => app,
        Ok(None) => {
            return HttpResponse::NotFound().json(serde_json::json!({
                "error": "Application not found"
            }));
        },
        Err(_) => {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to fetch application"
            }));
        }
    };

    if app.organization_id != org.id.unwrap() {
        return HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Application does not belong to the organization"
        }));
    }

    let ws = WebSocketActor::new(org.id.unwrap(), app.id.unwrap());
    
    // Use WsResponseBuilder to create the WebSocket connection
    match ws::WsResponseBuilder::new(ws, &req, stream).start_with_addr() {
        Ok((addr, response)) => {  // Note: swapped the order here - addr comes first
            // Add the connection to the WebSocket server
            websocket_server
                .add_connection(org.id.unwrap(), app.id.unwrap(), addr)
                .await;
            // Return the WebSocket response
            response
        }
        Err(e) => {
            log::error!("Failed to start WebSocket: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}