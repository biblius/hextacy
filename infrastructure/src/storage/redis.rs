use super::DatabaseError;
use crate::config;
use r2d2_redis::{
    r2d2::{Pool, PooledConnection},
    redis::{Client, ConnectionInfo, IntoConnectionInfo},
    RedisConnectionManager,
};
use tracing::trace;

pub type RedisPool = Pool<r2d2_redis::RedisConnectionManager>;
pub type RedisPoolConnection = PooledConnection<r2d2_redis::RedisConnectionManager>;

pub fn build_pool() -> RedisPool {
    let pool_size = config::get_or_default("RD_POOL_SIZE", "8")
        .parse::<u32>()
        .expect("Unable to parse RD_POOL_SIZE, maker sure it is a valid integer");

    let conn_info = connection_info();

    trace!("Building Redis pool for {:?}", conn_info.addr);

    let manager = RedisConnectionManager::new(conn_info)
        .expect("Error while attempting to construct Redis connection manager");

    Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .unwrap_or_else(|e| panic!("Failed to create redis pool: {}", e))
}

/// Generates a `ConnectionInfo` struct using the following environment variables:
///
/// `REDIS_URL`,
/// `RD_USER`,
/// `RD_PASSWORD`,
/// `RD_DATABASE`
///
/// Panics if it can't find any of the listed env variables apart from `RD_USE_DB` which defaults to 0.
fn connection_info() -> ConnectionInfo {
    let mut params = config::get_multiple(&["REDIS_URL", "RD_USER", "RD_PASSWORD", "RD_DATABASE"]);

    let db = params.pop().map_or_else(
        || {
            trace!("RD_DATABASE parameter not set, defaulting to 0");
            0
        },
        |s| {
            s.parse::<i64>()
                .expect("Invalid RD_DATABASE, make sure it's a valid integer")
        },
    );

    let password = params.pop().expect("RD_PASSWORD must be set");

    let username = params.pop().expect("RD_USER must be set");

    let db_url = params.pop().expect("REDIS_URL must be set");

    trace!("Buildig Redis connection info with {}", db_url);

    let mut conn_info = db_url.into_connection_info().unwrap();
    conn_info.username = Some(username);
    conn_info.passwd = Some(password);
    conn_info.db = db;

    conn_info
}

#[derive(Clone)]
pub struct Rd {
    pool: RedisPool,
}

impl Default for Rd {
    fn default() -> Self {
        Self::new()
    }
}

impl Rd {
    pub fn new() -> Self {
        Self { pool: build_pool() }
    }

    pub fn connect(&self) -> Result<RedisPoolConnection, DatabaseError> {
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            Err(e) => Err(DatabaseError::RdPoolConnection(e.to_string())),
        }
    }

    pub fn connect_direct() -> Result<Client, DatabaseError> {
        let db_url = config::get("REDIS_URL").expect("REDIS_URL must be set");
        match Client::open(db_url) {
            Ok(conn) => Ok(conn),
            Err(e) => Err(DatabaseError::RdDirectConnection(e)),
        }
    }
}
