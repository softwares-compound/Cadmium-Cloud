use crate::db::MongoRepo;
use crate::models::user::User;
use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage, HttpResponse,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use std::{
    env,
    sync::Arc,
    task::{Context, Poll},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String, // User email
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
        let db = req.app_data::<web::Data<MongoRepo>>().cloned(); // ✅ Fetch DB here

        Box::pin(async move {
            if let Some(cookie) = req.cookie("auth_token") {
                let token = cookie.value();
                println!("Token: {}", token);
                let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
                let validation = Validation::default();

                match decode::<Claims>(
                    token,
                    &DecodingKey::from_secret(secret.as_ref()),
                    &validation,
                ) {
                    Ok(token_data) => {
                        let user_email = token_data.claims.sub;

                        if let Some(db) = db {
                            let collection = db.db.collection::<User>("users");
                            match collection
                                .find_one(doc! { "email": &user_email }, None)
                                .await
                            {
                                Ok(Some(user)) => {
                                    req.extensions_mut().insert(user);
                                    return srv.call(req).await;
                                }
                                Ok(None) => {
                                    return Ok(req.into_response(
                                        HttpResponse::Unauthorized().json(
                                            serde_json::json!({ "message": "User not found" }),
                                        ),
                                    ));
                                }
                                Err(_) => {
                                    return Ok(req.into_response(
                                        HttpResponse::InternalServerError().json(
                                            serde_json::json!({ "message": "Database error" }),
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                    Err(_) => {
                        return Ok(req
                            .into_response(HttpResponse::Unauthorized().json(
                                serde_json::json!({ "message": "Invalid or expired token" }),
                            )));
                    }
                }
            }

            Ok(req.into_response(
                HttpResponse::Unauthorized()
                    .json(serde_json::json!({ "message": "Missing authentication token" })),
            ))
        })
    }
}
