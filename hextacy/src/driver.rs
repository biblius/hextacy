#[cfg(feature = "redis")]
pub mod cache;
#[cfg(feature = "db")]
pub mod db;
#[cfg(feature = "email")]
pub mod email;

use async_trait::async_trait;
use deadpool_redis::PoolError;
use thiserror::Error;

#[async_trait]
/// Trait used by data sources for establishing connections. Service components can bind their driver fields to this trait
/// in order for them obtain access to a generic connection.
pub trait Driver {
    type Connection;
    async fn connect(&self) -> Result<Self::Connection, DriverError>;
}

#[derive(Debug, Error)]
pub enum DriverError {
    #[cfg(any(feature = "full", feature = "db", feature = "mongo"))]
    #[error("Mongo driver error: {0}")]
    Mongo(#[from] mongodb::error::Error),

    #[cfg(any(feature = "full", feature = "db", feature = "postgres-diesel"))]
    #[error("Postgres pool error: {0}")]
    DieselConnection(#[from] r2d2::Error),

    #[cfg(any(feature = "full", feature = "db", feature = "postgres-diesel"))]
    #[error("Diesel error: {0}")]
    DieselResult(#[from] diesel::result::Error),

    #[cfg(any(feature = "full", feature = "db", feature = "redis"))]
    #[error("Redis pool error: {0}")]
    RedisConnection(#[from] PoolError),

    #[cfg(feature = "email")]
    #[error("Transport error: {0}")]
    Transport(#[from] lettre::transport::smtp::Error),

    #[cfg(feature = "email")]
    #[error("Email error: {0}")]
    Email(#[from] lettre::error::Error),

    #[cfg(any(feature = "db", feature = "full", feature = "postgres-seaorm"))]
    #[error("SeaORM Error: {0}")]
    SeaormConnection(#[from] sea_orm::DbErr),
}
