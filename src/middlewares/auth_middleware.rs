use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{
    env,
    task::{Context, Poll},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    // 🔹 Add `pub`
    pub sub: String, // User ID or email
    pub exp: usize,  // Expiration timestamp
}

pub struct AuthMiddleware;

impl<S> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Arc::new(service),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Arc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        Box::pin(async move {
            if let Some(cookie) = req.cookie("auth_token") {
                let token = cookie.value();
                print!("🔹 JWT: {} ====>>>>", token);
                let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
                let validation = Validation::default();

                match decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(secret.as_ref()),
                    &validation,
                ) {
                    Ok(token_data) => {
                        let claims = token_data.claims;

                        // ✅ Pass user details to next handler
                        req.extensions_mut().insert(claims);

                        return srv.call(req).await;
                    }
                    Err(err) => {
                        log::warn!("JWT Decoding Error: {:?}", err);
                        return Ok(req.into_response(
                            HttpResponse::Unauthorized().body("Invalid or expired token"),
                        ));
                    }
                }
            }

            log::warn!("No auth_token cookie found");
            Ok(
                req.into_response(
                    HttpResponse::Unauthorized().body("Missing authentication token"),
                ),
            )
        })
    }
}
