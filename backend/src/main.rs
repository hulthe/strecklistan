#![feature(specialization)]
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate diesel;

mod database;
pub mod models;
pub mod routes;
mod schema;
pub mod util;
pub mod auth;

use crate::database::create_pool;
use crate::database::DatabasePool;
use crate::models::user::JWTConfig;
use crate::routes::{graphql, session};
use crate::util::catchers::catchers;
use chrono::Duration;
use diesel_migrations::{
    find_migrations_directory, mark_migrations_in_directory, run_pending_migrations, setup_database,
};
use dotenv::dotenv;
use frank_jwt::Algorithm;
use rocket::routes;
use std::env;

fn handle_migrations(db_pool: &DatabasePool) {
    let run_migrations = env::var("RUN_MIGRATIONS")
        .map(|s| {
            s.parse().expect(&format!(
                "Could not parse \"{}\" as a bool for RUN_MIGRATIONS",
                s
            ))
        })
        .unwrap_or(false);

    if run_migrations {
        let connection = db_pool.get().expect("Could not connect to database");

        setup_database(&connection).expect("Could not set up database");

        let migrations_dir =
            find_migrations_directory().expect("Could not find migrations directory");

        let migrations = mark_migrations_in_directory(&connection, &migrations_dir)
            .expect("Could not get database migrations");

        if migrations.len() > 0 {
            println!("Migrations:");
            for (migration, applied) in migrations {
                println!(
                    "  [{}] {}",
                    if applied { "X" } else { " " },
                    migration
                        .file_path()
                        .and_then(|p| p.file_name())
                        .map(|p| p.to_string_lossy())
                        .unwrap_or("".into()),
                );
            }
        } else {
            eprintln!(
                "No database migrations available in \"{}\".",
                migrations_dir.to_string_lossy()
            );
        }

        run_pending_migrations(&connection).expect("Could not run database migrations");
    }
}

fn main() {
    dotenv().ok();

    let db_pool = create_pool().expect("Could not create database pool");

    handle_migrations(&db_pool);

    let jwt_config = JWTConfig {
        algorithm: Algorithm::HS512,
        secret: env::var("JWT_SECRET").expect("JWT_SECRET not set"),
        token_lifetime: Duration::weeks(2),
    };

    let mut rocket = rocket::ignite()
        .manage(db_pool)
        .manage(graphql::create_schema())
        .manage(jwt_config)
        .register(catchers())
        .mount(
            "/",
            routes![
                session::user_info,
                session::no_user,
                session::login,
                graphql::graphiql,
                graphql::post_graphql_handler_auth,
                graphql::post_graphql_handler,
            ],
        );
    let config = rocket.config();

    // Mount dev-only routes
    if config.environment.is_dev() {
        rocket = rocket.mount("/", routes![session::register]);
    }

    rocket.launch();
}
