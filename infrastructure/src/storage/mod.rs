pub mod mongo;
pub mod postgres;
pub mod redis;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Postgres pool error: {0}")]
    PgPoolConnection(String),
    #[error("Redis pool error: {0}")]
    RdPoolConnection(String),
    #[error("PG Connection error: {0}")]
    PgDirectConnection(#[from] diesel::ConnectionError),
    #[error("RD Connection error: {0}")]
    RdDirectConnection(#[from] r2d2_redis::redis::RedisError),
    #[error("Diesel error: {0}")]
    DieselResult(#[from] diesel::result::Error),
    #[error("Serde Error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Does not exist: {0}")]
    DoesNotExist(String),
}
