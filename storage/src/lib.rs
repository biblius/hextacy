pub mod mongo;
pub mod postgres;
pub mod redis;
pub use tracing::{info, warn};

pub enum DatabaseError {
    PgPoolConnection(String),
    RdPoolConnection(String),
    PgDirectConnection(diesel::ConnectionError),
    RdDirectConnection(r2d2_redis::redis::RedisError),
}

impl From<diesel::ConnectionError> for DatabaseError {
    fn from(e: diesel::ConnectionError) -> Self {
        Self::PgDirectConnection(e)
    }
}

impl From<r2d2_redis::redis::RedisError> for DatabaseError {
    fn from(e: r2d2_redis::redis::RedisError) -> Self {
        Self::RdDirectConnection(e)
    }
}
