use actix::{Actor, Handler,ActorContext, Message};
use actix_web_actors::ws;
use mongodb::bson::oid::ObjectId;
use actix::StreamHandler;
use actix::AsyncContext;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendLogId {
    pub log_id: ObjectId,
    pub app_id: ObjectId,
}

/// Represents a WebSocket connection.
pub struct WebSocketActor {
    pub organization_id: ObjectId,
}

impl WebSocketActor {
    pub fn new(organization_id: ObjectId) -> Self {
        Self { organization_id }
    }
}


impl Actor for WebSocketActor {
    type Context = ws::WebsocketContext<Self>;
}

impl Handler<SendLogId> for WebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: SendLogId, ctx: &mut Self::Context) {
        let message = format!("New log ID: {} and App ID: {}", msg.log_id,msg.app_id);
        ctx.text(message);
    }
}




impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => (),
            Ok(ws::Message::Text(text)) => {
                log::info!("Received message: {}", text);
            }
            Ok(ws::Message::Close(reason)) => {
                log::info!("WebSocket connection closing: {:?}", reason);

                // Get the current actor's address
                let conn = ctx.address();
                let org_id = self.organization_id.clone();

                // Call remove_connection asynchronously
                let websocket_server = crate::websocket::server::WebSocketServer::new(); 
                actix::spawn(async move {
                    websocket_server.remove_connection(org_id, conn).await;
                });

                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}
