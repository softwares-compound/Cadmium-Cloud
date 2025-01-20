use crate::handlers::user_handler;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/users").route("/signup", web::post().to(user_handler::signup)));
}
