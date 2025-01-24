use crate::db::MongoRepo;
use crate::models::otp::OtpEntry;
use chrono::Utc;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use rand::Rng;

pub async fn generate_otp(email: &str, db: &MongoRepo) -> String {
    let otp: String = rand::thread_rng().gen_range(100000..999999).to_string();
    let collection = db.db.collection::<OtpEntry>("otps");

    let otp_entry = OtpEntry {
        id: ObjectId::new(),
        email: email.to_string(),
        otp: otp.clone(),
        created_at: Utc::now(),
    };

    collection.insert_one(otp_entry, None).await.unwrap();
    otp
}

pub async fn verify_otp(email: &str, otp: &str, db: &MongoRepo) -> bool {
    let collection = db.db.collection::<OtpEntry>("otps");

    if let Some(stored_otp) = collection
        .find_one(doc! { "email": email, "otp": otp }, None)
        .await
        .unwrap()
    {
        collection
            .delete_one(doc! { "_id": stored_otp.id }, None)
            .await
            .unwrap(); // Remove OTP after successful verification
        return true;
    }
    false
}
