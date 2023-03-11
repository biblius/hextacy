use super::ClientError;
use async_trait::async_trait;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection, State},
    Connection, PgConnection,
};
use tracing::{info, trace};

use super::DBConnect;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPoolConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// Builds a postgres connection pool. Searches the shell env for `POSTGRES_URL` and `PG_POOL_SIZE`.
/// Panics if the db url isn't present or if the pool size is not parseable. The pool size defaults to 8 if not set.
pub fn build_pool(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    db: &str,
    pool_size: u32,
) -> PgPool {
    let url = format!("postgresql://{user}:{password}@{host}:{port}/{db}");

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

#[async_trait(?Send)]
impl DBConnect for Postgres {
    type Connection = PgPoolConnection;

    async fn connect(&self) -> Result<Self::Connection, ClientError> {
        trace!("Postgres - Attempting pooled connection");
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            Err(e) => Err(ClientError::PgPoolConnection(e.to_string())),
        }
    }
}

impl Postgres {
    pub fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        db: &str,
        pool_size: u32,
    ) -> Self {
        info!("Intitializing Postgres pool");
        Self {
            pool: build_pool(host, port, user, password, db, pool_size),
        }
    }

    /// Attempts to establish a pooled connection.
    pub fn connect(&self) -> Result<PgPoolConnection, ClientError> {
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            Err(e) => Err(ClientError::PgPoolConnection(e.to_string())),
        }
    }

    /// Expects a url as postgresql://user:password@host:port/database
    pub fn connect_direct(&self, db_url: &str) -> Result<PgConnection, ClientError> {
        PgConnection::establish(&db_url).map_err(Into::into)
    }

    /// Returns the state of the pool
    pub fn health_check(&self) -> State {
        self.pool.state()
    }
}
