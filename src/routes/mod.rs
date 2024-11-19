use actix_web::web;

mod logs;
mod health;

pub fn init(cfg: &mut web::ServiceConfig) {
    logs::init_routes(cfg);
    health::init_routes(cfg);
    
}
