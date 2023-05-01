pub use redis;

use crate::drivers::DriverError;
use r2d2::{Pool, PooledConnection};
use redis::{Client, ConnectionInfo, IntoConnectionInfo};
use tracing::{info, trace};

pub type RedisPool = Pool<Client>;
pub type RedisPoolConnection = PooledConnection<redis::Client>;

/// Contains a redis connection pool. An instance of this should be shared through the app with arcs.
#[derive(Debug, Clone)]
pub struct Redis {
    pool: RedisPool,
}

impl Redis {
    pub fn new(
        host: &str,
        port: u16,
        user: Option<&str>,
        password: Option<&str>,
        db: i64,
        pool_size: u32,
    ) -> Self {
        info!("Initializing redis pool");
        Self {
            pool: build_pool(host, port, user, password, db, pool_size),
        }
    }

    pub fn connect(&self) -> Result<RedisPoolConnection, DriverError> {
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            Err(e) => Err(DriverError::RdPoolConnection(e.to_string())),
        }
    }

    /// Expect a url as redis://username:password@host:port
    pub fn connect_direct(db_url: &str) -> Result<Client, DriverError> {
        match Client::open(db_url) {
            Ok(conn) => Ok(conn),
            Err(e) => Err(DriverError::RdDirectConnection(e)),
        }
    }
}

/// Builds a Redis connection pool with a default size of 8 workers
pub fn build_pool(
    host: &str,
    port: u16,
    user: Option<&str>,
    password: Option<&str>,
    db: i64,
    pool_size: u32,
) -> RedisPool {
    let conn_info = connection_info(host, port, user, password, db);

    trace!("Building Redis pool for {:?}", conn_info.addr);

    let driver = Client::open(conn_info).expect("Could not create redis driver");

    Pool::builder()
        .max_size(pool_size)
        .build(driver)
        .unwrap_or_else(|e| panic!("Failed to create redis pool: {e}"))
}

/// Panics if the DB url cannot be constructed
fn connection_info(
    host: &str,
    port: u16,
    user: Option<&str>,
    password: Option<&str>,
    db: i64,
) -> ConnectionInfo {
    let db_url = format!("redis://{host}:{port}");
    let mut conn_info = db_url.into_connection_info().unwrap();
    conn_info.redis.password = password.map(|pw| pw.to_string());
    conn_info.redis.username = user.map(|uname| uname.to_string());
    conn_info.redis.db = db;
    conn_info
}
