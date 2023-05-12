pub mod cache;
pub mod db;
#[cfg(feature = "email")]
pub mod email;

use async_trait::async_trait;
use deadpool_redis::PoolError;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug)]
/// A struct that contains a generic driver `A` that, through [Connect], establishes a database connection `C`.
/// Serves as a wrapper around connections so they can stay generic and consistent while building repositories.
///
/// Service adapters utilise this for establishing connections with a uniform API. One may implement this manually or
/// use the macros provided in `hextacy_macros` for quick implementations. The derive macros generate this struct
/// internally.
pub struct Driver<A, C>
where
    A: Connect<Connection = C>,
{
    pub driver: Arc<A>,
}

impl<A, C> Clone for Driver<A, C>
where
    A: Connect<Connection = C>,
{
    fn clone(&self) -> Self {
        Self {
            driver: self.driver.clone(),
        }
    }
}

impl<A, C> Driver<A, C>
where
    A: Connect<Connection = C>,
{
    pub fn new(driver: Arc<A>) -> Self {
        Self { driver }
    }
}

#[async_trait]
/// Trait used by drivers for establishing database connections. The [Driver] implements this and delegates
/// the `connect` method to any concrete type that gets instantiated in it.
pub trait Connect {
    type Connection;
    async fn connect(&self) -> Result<Self::Connection, DriverError>;
}

#[async_trait]
impl<A, C> Connect for Driver<A, C>
where
    A: Connect<Connection = C> + Send + Sync,
{
    type Connection = C;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        self.driver.connect().await
    }
}

#[derive(Debug, Error)]
pub enum DriverError {
    #[cfg(any(feature = "full", feature = "db", feature = "mongo"))]
    #[error("Mongo driver error: {0}")]
    Mongo(#[from] mongodb::error::Error),

    #[cfg(any(feature = "full", feature = "db", feature = "postgres-diesel"))]
    #[error("Postgres pool error: {0}")]
    DieselConnection(String),

    #[cfg(any(feature = "full", feature = "db", feature = "postgres-diesel"))]
    #[error("Diesel error: {0}")]
    DieselResult(#[from] diesel::result::Error),

    #[cfg(any(feature = "full", feature = "db", feature = "postgres-diesel"))]
    #[error("PG Connection error: {0}")]
    PgDirectConnection(#[from] diesel::ConnectionError),

    #[cfg(any(feature = "full", feature = "db", feature = "redis"))]
    #[error("Redis pool error: {0}")]
    RedisConnection(PoolError),

    #[cfg(any(feature = "full", feature = "db", feature = "redis"))]
    #[error("RD Connection error: {0}")]
    RdDirectConnection(#[from] redis::RedisError),

    #[cfg(feature = "email")]
    #[error("Transport error: {0}")]
    Transport(#[from] lettre::transport::smtp::Error),

    #[cfg(feature = "email")]
    #[error("Email error: {0}")]
    Email(#[from] lettre::error::Error),
}
