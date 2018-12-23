#![feature(plugin)]
#![feature(specialization)]
#![feature(proc_macro_hygiene, decl_macro)]
// Disable warnings caused by nightly rust phasing out this feature
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
extern crate diesel_migrations;
extern crate r2d2;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate juniper;
extern crate chrono;
extern crate dotenv;
extern crate hex;
extern crate juniper_rocket;
extern crate orion;

mod database;
pub mod models;
pub mod routes;
mod schema;
pub mod util;

use database::create_pool;
use diesel_migrations::{run_pending_migrations, setup_database};
use dotenv::dotenv;
use routes::{graphql, session};
use std::env;
use util::catchers::catchers;

fn main() {
    dotenv().ok();

    let db_pool = create_pool().expect("Could not create database pool");

    let run_migrations = env::var("RUN_MIGRATIONS")
        .map(|s| s.parse().unwrap_or(false))
        .unwrap_or(false);
    if run_migrations {
        let connection = db_pool.get().expect("Could not connect to database");

        setup_database(&connection).expect("Could not set up database");

        run_pending_migrations(&connection).expect("Could not run database migrations");
    }

    rocket::ignite()
        .manage(db_pool)
        .manage(graphql::create_schema())
        .register(catchers())
        .mount(
            "/",
            routes![
                session::user_info,
                session::no_user,
                session::login,
                session::register,
                graphql::graphiql,
                graphql::post_graphql_handler_auth,
                graphql::post_graphql_handler,
            ],
        )
        .launch();
}
