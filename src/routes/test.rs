use crate::handlers::test_handler;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/dashboard").route("/test", web::post().to(test_handler::test_handler)),
    );
}
