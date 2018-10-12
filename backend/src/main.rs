#![feature(custom_derive)]
#![feature(plugin)]
#![feature(specialization)]
#![feature(extern_prelude)]
#![plugin(rocket_codegen)]

// Disable warnings caused by nightly rust phasing out this feature
#![allow(proc_macro_derive_resolution_fallback)]

extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
extern crate diesel_migrations;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;
extern crate chrono;

mod schema;
mod database;
pub mod models;
pub mod util;
pub mod routes;

use std::env;
use dotenv::dotenv;
use rocket::Request;
use rocket::http::Status;
use diesel_migrations::{setup_database, run_pending_migrations};
use database::establish_connection;
use util::ErrorJson;
use routes::{
    event,
    signup,
};


#[catch(404)]
pub fn not_found(_: &Request) -> ErrorJson { ErrorJson {
    status: Status::NotFound.into(),
    description: "Route Not Found".into(),
}}


fn main() {
    dotenv().ok();

    let run_migrations = env::var("RUN_MIGRATIONS")
        .map(|s| s.parse().unwrap_or(false))
        .unwrap_or(false);
    if run_migrations {
        let connection = establish_connection()
            .expect("Could not connect to database");

        setup_database(&connection)
            .expect("Could not set up database");

        run_pending_migrations(&connection)
            .expect("Could not run database migrations");
    }

    rocket::ignite()
        .catch(catchers![
            not_found,
        ])
        .mount("/", routes![
               event::get_events,
               event::get_event,
               event::post_event,
               signup::get_event_signups,
               signup::get_signup,
               signup::post_signup,
        ]).launch();
}
