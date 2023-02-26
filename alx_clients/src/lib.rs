pub mod db;
#[cfg(feature = "email")]
pub mod email;
pub mod oauth;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Environment error: {0}")]
    Env(#[from] std::env::VarError),

    #[cfg(feature = "postgres")]
    #[error("Postgres pool error: {0}")]
    PgPoolConnection(String),
    #[cfg(feature = "postgres")]
    #[error("Diesel error: {0}")]
    DieselResult(#[from] diesel::result::Error),
    #[cfg(feature = "postgres")]
    #[error("PG Connection error: {0}")]
    PgDirectConnection(#[from] diesel::ConnectionError),

    #[cfg(feature = "redis")]
    #[error("Redis pool error: {0}")]
    RdPoolConnection(String),
    #[cfg(feature = "redis")]
    #[error("RD Connection error: {0}")]
    RdDirectConnection(#[from] r2d2_redis::redis::RedisError),

    #[cfg(feature = "email")]
    #[error("Transport error: {0}")]
    Transport(#[from] lettre::transport::smtp::Error),
    #[cfg(feature = "email")]
    #[error("Email error: {0}")]
    Email(#[from] lettre::error::Error),
}
