use crate::db::MongoRepo;
use crate::models::organization::Organization;
use actix_web::{web, HttpResponse, Responder,HttpRequest};
use mongodb::bson::oid::ObjectId;
use mongodb::bson::doc;
use serde::Deserialize;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrganization {
    pub org_name: String,
    pub admin_email: String,
    pub admin_password: String,
}

pub async fn create_organization(
    payload: web::Json<CreateOrganization>,
    data: web::Data<MongoRepo>,
) -> impl Responder {
    let existing_org = data
        .find_one_organization(doc! {
            "$or": [
                { "admin_email": &payload.admin_email },
                { "org_name": &payload.org_name }
            ]
        })
        .await;

    if let Ok(Some(_)) = existing_org {
        return HttpResponse::Conflict().json(serde_json::json!({
            "message": "An organization with the provided email or name already exists.",
        }));
    }

    let org = Organization {
        id: Some(ObjectId::new()),
        org_name: payload.org_name.clone(),
        admin_email: payload.admin_email.clone(),
        admin_password: payload.admin_password.clone(),
        cd_id: generate_unique_id(),
        cd_secret: generate_unique_id(),
    };

    match data.create_organization(org).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Organization created successfully",
        })),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

/// Generates a unique identifier for `cd_id` and `cd_secret`.
fn generate_unique_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32) // Generate a 32-character random string
        .map(char::from)
        .collect()
}

pub async fn get_organization_details(
    req: HttpRequest,
    data: web::Data<MongoRepo>,
) -> impl Responder {
    // Extract CD-ID and CD-SECRET headers
    let cd_id = req
        .headers()
        .get("CD-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let cd_secret = req
        .headers()
        .get("CD-SECRET")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();

    // Find organization by CD-ID and CD-SECRET
    let collection = data.db.collection::<mongodb::bson::Document>("organizations");
    let filter = doc! { "cd_id": cd_id, "cd_secret": cd_secret };

    match collection.find_one(filter, None).await {
        Ok(Some(org)) => {
            let id = org.get_object_id("_id").ok();
            let org_name = org.get_str("org_name").ok();
            if let (Some(id), Some(org_name)) = (id, org_name) {
                HttpResponse::Ok().json(serde_json::json!({
                    "id": id.to_hex(),
                    "org_name": org_name,
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Incomplete organization data",
                }))
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid CD-ID or CD-SECRET",
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to fetch organization details",
        })),
    }
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub admin_email: String,
    pub admin_password: String,
}

pub async fn login(
    payload: web::Json<LoginPayload>,
    data: web::Data<MongoRepo>,
) -> impl Responder {
    let collection = data.db.collection::<Organization>("organizations");
    let filter = doc! {
        "admin_email": &payload.admin_email,
        "admin_password": &payload.admin_password,
    };

    match collection.find_one(filter, None).await {
        Ok(Some(org)) => {
            let response = serde_json::json!({
                "organization_name": org.org_name,
                "id": org.id.unwrap().to_hex(),
                "cd_id": org.cd_id,
                "cd_secret": org.cd_secret,
            });
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid email or password",
        })),
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to authenticate organization",
        })),
    }
}