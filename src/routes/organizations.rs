use crate::handlers::organization_handler;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/organizations")
            .route(
                "",
                web::post().to(organization_handler::create_organization),
            )
            .route(
                "",
                web::get().to(organization_handler::get_organization_details),
            ), // Add other routes like update, delete
    );
}
