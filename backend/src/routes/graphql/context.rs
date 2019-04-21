use crate::auth::AuthScope;
use crate::database::DatabasePool;
use crate::models::User;

pub struct Context {
    pub pool: DatabasePool,
    user: Option<User>,
}

impl Context {
    pub fn new(pool: DatabasePool, user: Option<User>) -> Context {
        Context { pool, user }
    }

    pub fn get_auth(&self, scope: AuthScope) -> Result<&User, String> {
        self.user
            .as_ref()
            .ok_or_else(|| format!("Authentication required for resource '{}'", scope.to_str()))
    }
}

impl juniper::Context for Context {}
