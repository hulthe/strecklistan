use crate::database::{create_pool, DatabasePool};
use crate::schema::tables::event_signups;
use crate::schema::tables::events;
use crate::schema::tables::users;
use crate::Opt;
use clap::Parser;
use diesel::RunQueryDsl;
use dotenv::dotenv;

pub struct DatabaseState {
    db_pool: DatabasePool,
}

impl DatabaseState {
    pub fn new() -> (DatabaseState, DatabasePool) {
        dotenv().ok();
        let opt = Opt::parse();
        let db_pool = create_pool(&opt).expect("Could not create database pool");
        let state = DatabaseState {
            db_pool: db_pool.clone(),
        };
        (state, db_pool)
    }
}

impl Drop for DatabaseState {
    fn drop(&mut self) {
        let connection = self
            .db_pool
            .get()
            .expect("Could not get database connection");
        diesel::delete(events::table)
            .execute(&connection)
            .expect("Could not truncate testing database table");
        diesel::delete(event_signups::table)
            .execute(&connection)
            .expect("Could not truncate testing database table");
        diesel::delete(users::table)
            .execute(&connection)
            .expect("Could not truncate testing database table");
    }
}
