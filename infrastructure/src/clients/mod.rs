pub mod email;
pub mod storage;

use thiserror::Error;

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
    #[error("Lettre Error: {0}")]
    Lettre(#[from] lettre::error::Error),
    #[error("SMTP Error: {0}")]
    SmtpTransport(#[from] lettre::transport::smtp::Error),
}
