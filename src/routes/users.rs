use crate::handlers::user_handler;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/verify_email", web::post().to(user_handler::verify_email))
            .route(
                "/signup",
                web::post().to(user_handler::verify_otp_and_signup),
            )
            .route("/logout", web::post().to(user_handler::logout)),
    );
}
