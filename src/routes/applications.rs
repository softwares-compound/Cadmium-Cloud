use actix_web::web;
use crate::handlers::application_handler;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/applications")
            .route("", web::post().to(application_handler::create_application))
            // Add other routes like get, update, delete
    );
}
