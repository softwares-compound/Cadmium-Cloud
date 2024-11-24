use crate::handlers::log_handler;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/logs")
            .route("", web::get().to(log_handler::get_all_logs))
            .route("/{log_id}", web::get().to(log_handler::get_log_by_id))
            .route("", web::post().to(log_handler::save_log)),
    );
}
