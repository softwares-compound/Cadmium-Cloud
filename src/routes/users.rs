// src/routes/users.rs
use crate::handlers::user_handler;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/send_otp", web::post().to(user_handler::send_otp))
            .route(
                "/signup",
                web::post().to(user_handler::verify_otp_and_signup),
            ),
    );
}
