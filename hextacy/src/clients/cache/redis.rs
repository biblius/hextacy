pub use redis;

use crate::clients::ClientError;
use r2d2::{Pool, PooledConnection};
use redis::{Client, ConnectionInfo, IntoConnectionInfo};
use tracing::{info, trace};

pub type RedisPool = Pool<redis::Client>;
pub type RedisPoolConnection = PooledConnection<redis::Client>;

/// Builds a Redis connection pool with a default size of 8 workers
pub fn build_pool(
    host: &str,
    port: u16,
    user: &str,
    password: &str,
    db: i64,
    pool_size: u32,
) -> RedisPool {
    let conn_info = connection_info(host, port, user, password, db);

    trace!("Building Redis pool for {:?}", conn_info.addr);

    let client = redis::Client::open(conn_info).expect("Could not create redis client");

    Pool::builder()
        .max_size(pool_size)
        .build(client)
        .unwrap_or_else(|e| panic!("Failed to create redis pool: {e}"))
}

/// Panics if the DB url cannot be constructed
fn connection_info(host: &str, port: u16, user: &str, password: &str, db: i64) -> ConnectionInfo {
    let db_url = format!("redis://{user}:{password}@{host}:{port}");
    let mut conn_info = db_url.into_connection_info().unwrap();
    conn_info.redis.db = db;
    conn_info
}

#[derive(Debug, Clone)]
pub struct Redis {
    pool: RedisPool,
}

impl Redis {
    pub fn new(host: &str, port: u16, user: &str, password: &str, db: i64, pool_size: u32) -> Self {
        info!("Initializing redis pool");
        Self {
            pool: build_pool(host, port, user, password, db, pool_size),
        }
    }

    pub fn connect(&self) -> Result<RedisPoolConnection, ClientError> {
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            Err(e) => Err(ClientError::RdPoolConnection(e.to_string())),
        }
    }

    /// Expect a url as redis://username:password@host:port
    pub fn connect_direct(db_url: &str) -> Result<Client, ClientError> {
        match Client::open(db_url) {
            Ok(conn) => Ok(conn),
            Err(e) => Err(ClientError::RdDirectConnection(e)),
        }
    }
}
