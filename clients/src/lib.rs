use thiserror::Error;

pub mod db;
pub mod email;

#[derive(Debug, Error)]
pub enum ClientError {
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
    #[error("Transport error: {0}")]
    Transport(#[from] lettre::transport::smtp::Error),
    #[error("Email error: {0}")]
    Email(#[from] lettre::error::Error),
}
