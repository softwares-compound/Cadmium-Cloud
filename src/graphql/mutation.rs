use crate::db::MongoRepo;
use crate::models::user::User;
use async_graphql::{Context, Object, Result};
use mongodb::bson::doc;

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Signup a new user
    async fn signup(
        &self,
        ctx: &Context<'_>,
        first_name: String,
        last_name: String,
        email: String,
        password: String,
    ) -> Result<String> {
        let mongo_repo = ctx.data::<MongoRepo>()?;
        let collection = mongo_repo.db.collection::<User>("users");

        // Check if user already exists
        if collection
            .find_one(doc! { "email": &email }, None)
            .await?
            .is_some()
        {
            return Err("User already exists!".into());
        }

        // Hash the password and create user
        let user = User {
            id: None,
            first_name,
            last_name,
            email: email.clone(),
            password_hash: User::hash_password(&password),
        };

        // Insert user into MongoDB
        collection.insert_one(user, None).await?;

        Ok("User signed up successfully".to_string())
    }
}
