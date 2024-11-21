use actix_web::web;

mod applications;
mod health;
mod logs;
mod organizations;
mod websocket;

pub fn init(cfg: &mut web::ServiceConfig) {
    logs::init_routes(cfg);
    health::init_routes(cfg);
    organizations::init_routes(cfg);
    applications::init_routes(cfg);
    websocket::init_routes(cfg);
}
