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
extern crate chrono;
extern crate dotenv;
extern crate orion;
extern crate hex;

mod database;
pub mod models;
pub mod routes;
mod schema;
pub mod util;

use database::establish_connection;
use diesel_migrations::{run_pending_migrations, setup_database};
use dotenv::dotenv;
use routes::{session, event, signup};
use std::env;
use util::catchers;

fn main() {
    dotenv().ok();

    let run_migrations = env::var("RUN_MIGRATIONS")
        .map(|s| s.parse().unwrap_or(false))
        .unwrap_or(false);
    if run_migrations {
        let connection =
            establish_connection().expect("Could not connect to database");

        setup_database(&connection).expect("Could not set up database");

        run_pending_migrations(&connection).expect(
            "Could not run database migrations",
        );
    }

    rocket::ignite()
        .catch(catchers![
            catchers::not_found,
            catchers::unauthorized,
            catchers::bad_request,
        ])
        .mount(
            "/",
            routes![
                session::user_info,
                session::login,
                session::register,
                event::get_events,
                event::get_event,
                event::post_event,
                signup::get_event_signups,
                signup::get_signup,
                signup::post_signup,
            ],
        )
        .launch();
}
