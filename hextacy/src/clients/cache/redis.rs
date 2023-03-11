use crate::clients::ClientError;
use r2d2_redis::{
    r2d2::{Pool, PooledConnection},
    redis::{Client, ConnectionInfo, IntoConnectionInfo},
    RedisConnectionManager,
};
use tracing::{info, trace};

pub use r2d2_redis::redis::Commands;
pub use r2d2_redis::redis::FromRedisValue;
pub use r2d2_redis::redis::RedisError;
pub use r2d2_redis::redis::ToRedisArgs;

pub type RedisPool = Pool<r2d2_redis::RedisConnectionManager>;
pub type RedisPoolConnection = PooledConnection<r2d2_redis::RedisConnectionManager>;

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

    let manager = RedisConnectionManager::new(conn_info)
        .expect("Error while attempting to construct Redis connection manager");

    Pool::builder()
        .max_size(pool_size)
        .build(manager)
        .unwrap_or_else(|e| panic!("Failed to create redis pool: {e}"))
}

/// Panics if the DB url cannot be constructed
fn connection_info(host: &str, port: u16, user: &str, password: &str, db: i64) -> ConnectionInfo {
    let db_url = format!("redis://{host}:{port}");

    trace!("Building Redis connection info with {db_url}");

    let mut conn_info = db_url.into_connection_info().unwrap();
    conn_info.username = Some(user.to_string());
    conn_info.passwd = Some(password.to_string());
    conn_info.db = db;

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
