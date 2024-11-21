use crate::handlers::organization_handler;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/organizations").route(
            "",
            web::post().to(organization_handler::create_organization),
        ), // Add other routes like get, update, delete
    );
}
