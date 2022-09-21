use std::any::Any;

use super::DatabaseError;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    Connection, Insertable, PgConnection, Queryable,
};
use tracing::trace;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPoolConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// Builds a postgres connection pool. Searches the shell env for `POSTGRES_URL` and `PG_POOL_SIZE`.
/// Panics if the db url isn't present or if the pool size is not parseable. The pool size defaults to 8 if not set.
pub fn build_pool() -> PgPool {
    let mut params =
        config::get_or_default_multiple(vec![("POSTGRES_URL", ""), ("PG_POOL_SIZE", "8")]);

    let pool_size = params.pop().unwrap().parse::<u32>().unwrap();

    let db_url = params.pop().unwrap();

    assert!(!db_url.is_empty(), "POSTGRES_URL must be set");

    trace!("Bulding Postgres pool for {}", db_url);

    let manager = ConnectionManager::<PgConnection>::new(db_url);

    Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .unwrap_or_else(|e| panic!("Failed to create postgres pool: {}", e))
}

#[derive(Clone)]
pub struct Pg {
    pool: PgPool,
}

impl Default for Pg {
    fn default() -> Self {
        Self::new()
    }
}

impl Pg {
    pub fn new() -> Self {
        Self { pool: build_pool() }
    }

    /// Attempts to establish a pooled connection.
    pub fn connect(&self) -> Result<PgPoolConnection, DatabaseError> {
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            Err(e) => Err(DatabaseError::PgPoolConnection(e.to_string())),
        }
    }

    /// Attempts to establish a direct connection to the postgres server. Panics if `POSTGRES_URL` is not set
    /// in the environment.
    pub fn connect_direct() -> Result<PgConnection, DatabaseError> {
        let db_url = config::get("POSTGRES_URL").expect("POSTGRES_URL must be set");

        match PgConnection::establish(&db_url) {
            Ok(conn) => Ok(conn),
            Err(e) => Err(e.into()),
        }
    }

    /// Attempts a direct connection and tries to complete the transaction with the provided closure.
    ///
    pub fn transaction<F, R>(
        models: Vec<Box<dyn SqlModel>>,
        conn: &mut PgConnection,
        f: F,
    ) -> Result<R, DatabaseError>
    where
        F: FnOnce(Vec<Box<dyn SqlModel>>, &mut PgConnection) -> Result<R, diesel::result::Error>,
    {
        match conn.transaction(|conn| f(models, conn)) {
            Ok(r) => Ok(r),
            Err(e) => Err(e.into()),
        }
    }
}

pub trait SqlModel {
    fn as_any(&self) -> &dyn Any;
}

#[macro_export]
macro_rules! pg_transaction {
    ($($a: expr),+, $i: item) => {
        3
    };
}
