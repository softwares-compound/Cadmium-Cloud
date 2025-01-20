use crate::db::MongoRepo;
use crate::graphql::mutation::MutationRoot;
use crate::graphql::query::QueryRoot;
use async_graphql::{EmptySubscription, Schema};

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn create_schema(mongo_repo: MongoRepo) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(mongo_repo) // Share MongoRepo
        .finish()
}
