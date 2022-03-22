pub mod event;
pub mod transaction;

use crate::Opt;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{
    find_migrations_directory, mark_migrations_in_directory, run_pending_migrations, setup_database,
};
use r2d2::{Pool, PooledConnection};
use std::error::Error;

pub type DatabasePool = Pool<ConnectionManager<PgConnection>>;
pub type DatabaseConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create_pool(opt: &Opt) -> Result<DatabasePool, Box<dyn Error>> {
    let db_manager: ConnectionManager<PgConnection> = ConnectionManager::new(&opt.database);
    let db_pool: Pool<ConnectionManager<PgConnection>> =
        Pool::builder().max_size(15).build(db_manager)?;
    Ok(db_pool)
}

pub fn run_migrations(db_pool: &DatabasePool) {
    let connection = db_pool.get().expect("Could not connect to database");

    setup_database(&connection).expect("Could not set up database");

    let migrations_dir = find_migrations_directory().expect("Could not find migrations directory");

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
