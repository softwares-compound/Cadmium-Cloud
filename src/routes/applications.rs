use crate::handlers::application_handler;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/applications")
            .route("", web::post().to(application_handler::create_application))
            .route("", web::get().to(application_handler::get_applications))
            .route("/{application_id}", web::delete().to(application_handler::delete_application)), // Added delete route
    );
}
