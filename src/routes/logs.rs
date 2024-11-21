use crate::handlers::log_handler;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/logs", web::post().to(log_handler::save_log));
}
