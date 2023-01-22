use super::ClientError;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection, State},
    Connection, PgConnection,
};
use tracing::{info, trace};
use utils::env;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPoolConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// Builds a postgres connection pool. Searches the shell env for `POSTGRES_URL` and `PG_POOL_SIZE`.
/// Panics if the db url isn't present or if the pool size is not parseable. The pool size defaults to 8 if not set.
pub fn build_pool() -> PgPool {
    let mut params = env::get_multiple(&["POSTGRES_URL", "PG_POOL_SIZE"]);

    let pool_size = params
        .pop()
        .unwrap()
        .parse::<u32>()
        .expect("Invalid PG pool size");

    let db_url = params.pop().unwrap();

    trace!("Bulding Postgres pool for {}", db_url);

    let manager = ConnectionManager::<PgConnection>::new(db_url);

    Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .unwrap_or_else(|e| panic!("Failed to create postgres pool: {}", e))
}

#[derive(Debug, Clone)]
pub struct Postgres {
    pool: PgPool,
}

impl Default for Postgres {
    fn default() -> Self {
        Self::new()
    }
}

impl Postgres {
    pub fn new() -> Self {
        info!("Intitializing Postgres pool");
        Self { pool: build_pool() }
    }

    /// Attempts to establish a pooled connection.
    pub fn connect(&self) -> Result<PgPoolConnection, ClientError> {
        trace!("Postgres - Attempting pooled connection");
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            Err(e) => Err(ClientError::PgPoolConnection(e.to_string())),
        }
    }

    /// Attempts to establish a direct connection to the postgres server. Panics if `POSTGRES_URL` is not set
    /// in the environment.
    pub fn connect_direct() -> Result<PgConnection, ClientError> {
        let db_url = env::get("POSTGRES_URL").expect("POSTGRES_URL must be set");

        PgConnection::establish(&db_url).map_err(Into::into)
    }

    /// Returns the state of the pool
    pub fn health_check(&self) -> State {
        self.pool.state()
    }
}
