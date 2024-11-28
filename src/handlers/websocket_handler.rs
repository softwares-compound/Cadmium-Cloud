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

    if cd_id.is_none() || cd_secret.is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Missing CD-ID or CD-Secret headers"
        }));
    }

    let cd_id = cd_id.unwrap();
    let cd_secret = cd_secret.unwrap();

    // Authenticate organization using CD-ID and CD-Secret
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

    // Associate WebSocket with the organization instead of an application
    let ws = WebSocketActor::new(org.id.unwrap());

    match ws::WsResponseBuilder::new(ws, &req, stream).start_with_addr() {
        Ok((addr, response)) => {
            websocket_server.add_connection(org.id.unwrap(), addr).await;
            response
        }
        Err(e) => {
            log::error!("Failed to start WebSocket: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
