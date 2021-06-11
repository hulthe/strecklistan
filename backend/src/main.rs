#[macro_use]
extern crate diesel;

mod database;
pub mod models;
pub mod routes;
mod schema;
pub mod util;

use crate::database::create_pool;
use crate::database::DatabasePool;
use crate::routes::{index, rest};
use crate::util::{catchers, StaticCachedFiles};
use diesel_migrations::{
    find_migrations_directory, mark_migrations_in_directory, run_pending_migrations, setup_database,
};
use dotenv::dotenv;
use rocket::fs::FileServer;
use rocket::routes;
use std::env;

fn handle_migrations(db_pool: &DatabasePool) {
    let run_migrations = env::var("RUN_MIGRATIONS")
        .map(|s| {
            s.parse().unwrap_or_else(|_| {
                panic!("Could not parse \"{}\" as a bool for RUN_MIGRATIONS", s)
            })
        })
        .unwrap_or(false);

    if run_migrations {
        let connection = db_pool.get().expect("Could not connect to database");

        setup_database(&connection).expect("Could not set up database");

        let migrations_dir =
            find_migrations_directory().expect("Could not find migrations directory");

        let migrations = mark_migrations_in_directory(&connection, &migrations_dir)
            .expect("Could not get database migrations");

        if !migrations.is_empty() {
            println!("Migrations:");
            for (migration, applied) in migrations {
                println!(
                    "  [{}] {}",
                    if applied { "X" } else { " " },
                    migration
                        .file_path()
                        .and_then(|p| p.file_name())
                        .map(|p| p.to_string_lossy())
                        .unwrap_or_default()
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

#[rocket::main]
async fn main() {
    dotenv().ok();

    let db_pool = create_pool().expect("Could not create database pool");

    handle_migrations(&db_pool);

    let enable_static_file_cache: bool = env::var("ENABLE_STATIC_FILE_CACHE")
        .map(|s| {
            s.parse()
                .expect("Invalid ENABLE_STATIC_FILE_CACHE. Expected true or false.")
        })
        .unwrap_or(false);

    let max_age = env::var("STATIC_FILES_MAX_AGE")
        .map(|s| {
            s.parse()
                .expect("Invalid STATIC_FILES_MAX_AGE. Expected a number.")
        })
        .unwrap_or(0);

    let mut rocket = rocket::build()
        .manage(db_pool)
        .register("/", catchers())
        .mount(
            "/api/",
            routes![
                rest::event::get_event,
                rest::event::get_event_range,
                rest::inventory::get_inventory,
                rest::inventory::get_tags,
                rest::inventory::get_inventory_bundles,
                rest::transaction::get_transactions,
                rest::transaction::post_transaction,
                rest::transaction::delete_transaction,
                rest::book_account::get_accounts,
                rest::book_account::get_master_accounts,
                rest::book_account::add_account,
                rest::member::get_members,
                rest::member::add_member_with_book_account,
                rest::get_api_version,
                rest::izettle::izettle_bridge_poll::poll_for_transaction,
                rest::izettle::izettle_bridge_result::complete_izettle_transaction,
                rest::izettle::izettle_transaction::begin_izettle_transaction,
                rest::izettle::izettle_transaction_poll::poll_for_izettle,
            ],
        )
        .mount("/", routes![index::wildcard, index::root]);

    let static_routes = &[("/pkg", "www/pkg"), ("/static", "www/static")];

    for &(route, path) in static_routes {
        rocket = if enable_static_file_cache {
            rocket.mount(route, StaticCachedFiles::from(path).max_age(max_age))
        } else {
            rocket.mount(route, FileServer::from(path))
        };
    }

    rocket.launch().await.unwrap();
}
