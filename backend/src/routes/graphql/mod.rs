pub mod context;
mod event;
mod root;

use self::context::Context;
use self::root::{RootMutation, RootQuery};
use database::DatabasePool;
use juniper::RootNode;
use rocket::response::content;
use rocket::State;

pub type Schema = RootNode<'static, RootQuery, RootMutation>;

pub fn create_schema() -> Schema {
    Schema::new(RootQuery {}, RootMutation {})
}

#[get("/")]
pub fn graphiql() -> content::Html<String> {
    juniper_rocket::graphiql_source("/graphql")
}

#[post("/graphql", data = "<request>")]
pub fn post_graphql_handler(
    request: juniper_rocket::GraphQLRequest,
    schema: State<Schema>,
    db_pool: State<DatabasePool>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(&schema, &Context::new(db_pool.inner().clone(), None))
}
