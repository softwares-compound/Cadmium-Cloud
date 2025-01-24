use crate::handlers::forget_password_handler;
use crate::handlers::user_handler;
use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .route("/verify_email", web::post().to(user_handler::verify_email))
            .route(
                "/signup",
                web::post().to(user_handler::verify_and_delete_otp_and_signup),
            )
            .route("/logout", web::post().to(user_handler::logout))
            .route(
                "/forgot_password",
                web::post().to(forget_password_handler::send_reset_otp),
            )
            .route(
                "/verify_forgot_password",
                web::post().to(forget_password_handler::verify_forgot_password_otp),
            )
            .route(
                "/reset_password",
                web::post().to(forget_password_handler::reset_password),
            ),
    );
}
