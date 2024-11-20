use actix_web::web;

mod logs;
mod health;
mod organizations;
mod applications;

pub fn init(cfg: &mut web::ServiceConfig) {
    logs::init_routes(cfg);
    health::init_routes(cfg);
    organizations::init_routes(cfg);
    applications::init_routes(cfg);
}
