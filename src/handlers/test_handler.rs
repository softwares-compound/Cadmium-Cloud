use crate::middlewares::auth_middleware::Claims;
use actix_web::{HttpMessage, HttpRequest, Responder};

pub async fn test_handler(req: HttpRequest) -> impl Responder {
    // Retrieve user claims set by middleware
    if let Some(claims) = req.extensions().get::<Claims>() {
        format!("Welcome, {}!", claims.sub) // User ID from JWT
    } else {
        "Unauthorized".to_string()
    }
}
