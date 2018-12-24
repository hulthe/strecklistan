use crate::database::DatabasePool;
use crate::models::User;
use std::fmt::Display;

pub struct Context {
    pub pool: DatabasePool,
    user: Option<User>,
}

impl Context {
    pub fn new(pool: DatabasePool, user: Option<User>) -> Context {
        Context { pool, user }
    }

    pub fn get_auth<T: Display>(&self, resource_name: T) -> Result<&User, String> {
        self.user
            .as_ref()
            .ok_or_else(|| format!("Authentication required for resource '{}'", resource_name))
    }
}

impl juniper::Context for Context {}
