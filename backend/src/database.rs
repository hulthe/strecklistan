use std::env;
use std::error::Error;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::{Pool, PooledConnection};

pub type DatabasePool = Pool<ConnectionManager<PgConnection>>;
pub type DatabaseConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create_pool() -> Result<DatabasePool, Box<Error>> {
    let db_url =
        env::var("DATABASE_URL")?;
    let db_manager: ConnectionManager<PgConnection> = ConnectionManager::new(db_url);
    let db_pool: Pool<ConnectionManager<PgConnection>> = Pool::builder()
        .max_size(15)
        .build(db_manager)?;
    Ok(db_pool)
}
