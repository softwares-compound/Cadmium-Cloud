use async_graphql::{Schema, EmptyMutation, EmptySubscription};
use crate::graphql::query::QueryRoot;
use crate::db::MongoRepo;

pub type AppSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub fn create_schema(mongo_repo: MongoRepo) -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(mongo_repo) // Share the MongoRepo instance
        .finish()
}
