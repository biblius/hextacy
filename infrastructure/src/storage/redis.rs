use super::DatabaseError;
use crate::config::env;
use diesel::r2d2::State;
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
pub fn build_pool() -> RedisPool {
    let pool_size = env::get_or_default("RD_POOL_SIZE", "8")
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
/// Panics if it can't find any of the listed env variables apart from `RD_DATABASE` which defaults to 0.
fn connection_info() -> ConnectionInfo {
    let mut params = env::get_multiple(&["REDIS_URL", "RD_USER", "RD_PASSWORD", "RD_DATABASE"]);

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
        info!("Initializing redis pool");
        Self { pool: build_pool() }
    }

    pub fn connect(&self) -> Result<RedisPoolConnection, DatabaseError> {
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            Err(e) => Err(DatabaseError::RdPoolConnection(e.to_string())),
        }
    }

    pub fn connect_direct() -> Result<Client, DatabaseError> {
        let db_url = env::get("REDIS_URL").expect("REDIS_URL must be set");
        match Client::open(db_url) {
            Ok(conn) => Ok(conn),
            Err(e) => Err(DatabaseError::RdDirectConnection(e)),
        }
    }

    pub fn health_check(&self) -> State {
        self.pool.state()
    }
}

use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;

/// Utility struct
pub struct Cache;

impl Cache {
    pub fn get<T: DeserializeOwned>(
        cache_id: CacheId,
        key: &str,
        conn: &mut RedisPoolConnection,
    ) -> Result<T, DatabaseError> {
        let key = Self::prefix_id(cache_id, &key);
        let result = conn.get::<&str, String>(&key)?;
        trace!("Found redis key : {}", result);
        serde_json::from_str::<T>(&result).map_err(Into::into)
    }

    pub fn set<T: Serialize>(
        cache_id: CacheId,
        key: &str,
        val: &T,
        ex: Option<usize>,
        conn: &mut RedisPoolConnection,
    ) -> Result<(), DatabaseError> {
        let key = Self::prefix_id(cache_id, &key);
        let value = serde_json::to_string(&val)?;
        if let Some(ex) = ex {
            conn.set_ex::<&str, String, ()>(&key, value, ex)
                .map_err(Into::into)
        } else {
            conn.set::<&str, String, ()>(&key, value)
                .map_err(Into::into)
        }
    }

    pub fn delete(
        cache_id: CacheId,
        key: &str,
        conn: &mut RedisPoolConnection,
    ) -> Result<(), DatabaseError> {
        trace!("Deleting value under {}:{}", cache_id, key);
        conn.del::<String, ()>(Self::prefix_id(cache_id, &key))
            .map_err(Into::into)
    }

    pub fn prefix_id<T: ToRedisArgs + Display>(cache_id: CacheId, key: &T) -> String {
        format!("{}:{}", cache_id, key)
    }
}

#[derive(Debug)]
pub enum CacheId {
    LoginAttempts,
    Session,
    OTPToken,
    RegToken,
    PWToken,
}

impl Display for CacheId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheId::LoginAttempts => write!(f, "auth:login_attempts"),
            CacheId::OTPToken => write!(f, "auth:otp"),
            CacheId::Session => write!(f, "auth:session"),
            CacheId::RegToken => write!(f, "auth:registration_token"),
            CacheId::PWToken => write!(f, "auth:set_pw"),
        }
    }
}
