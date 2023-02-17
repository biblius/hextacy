use crate::ClientError;
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
    let url = env::get("POSTGRES_URL").expect("POSTGRES_URL must be set");
    let pool_size = env::get_or_default("PG_POOL_SIZE", "8")
        .parse::<u32>()
        .expect("Unable to parse PG_POOL_SIZE, maker sure it is a valid integer");

    trace!("Bulding Postgres pool for {url}");

    let manager = ConnectionManager::<PgConnection>::new(url);

    Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .unwrap_or_else(|e| panic!("Failed to create postgres pool: {e}"))
}

/// Contains a connection pool for postgres. An instance of this
/// should be shared through the app with Arcs
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
    pub fn connect_direct(&self) -> Result<PgConnection, ClientError> {
        let db_url = env::get("POSTGRES_URL").expect("POSTGRES_URL must be set");

        PgConnection::establish(&db_url).map_err(Into::into)
    }

    /// Returns the state of the pool
    pub fn health_check(&self) -> State {
        self.pool.state()
    }
}
