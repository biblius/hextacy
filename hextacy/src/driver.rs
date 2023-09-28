use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
/// Drivers are intended to provide a simple interface for establishing generic connections that other components
/// can use to remain decoupled from a concrete implementation. By utilising this trait, concrete data sources and clients
/// can easily be changed without interfering with the business layer.
/// A concrete implementation of a driver is usually a thin wrapper around a connection pool or a client.
///
/// The necessity for this trait arises from the fact that data sources have to pass around connections for
/// transactions. Since transactions could span multiple tables in the same execution flow, several repositories
/// must be able work on the same transaction. This is why repositories must take in a connection - the driver trait is here
/// to provide it. Services that utilise this trait can be bound to multiple repositories and one driver to provide the
/// connections for those repositories.
///
/// Check out the [adapters module][crate::adapters] to see concrete implementations.
///
/// The MVP of 6TC.
pub trait Driver {
    type Connection;
    async fn connect(&self) -> Result<Self::Connection, DriverError>;
}

/// Used for creating bounds on generic connections when the adapter needs to have atomic repository access.
///
/// This trait is used to normalise the API for transactions that are connection based and transactions that
/// return a transaction struct.
///
/// When transactions are connection based, the `TransactionResult` is typically
/// the connection on which the transaction is started.
///
/// When they are struct based, the adapter must implement a repository trait for both the
/// connection and transaction (usually a trait is provided for both so one can use it to
/// mitigate 2 different implementations).
#[async_trait]
pub trait Atomic: Sized {
    type TransactionResult: Send;

    async fn start_transaction(self) -> Result<Self::TransactionResult, DriverError>;
    async fn commit_transaction(tx: Self::TransactionResult) -> Result<(), DriverError>;
    async fn abort_transaction(tx: Self::TransactionResult) -> Result<(), DriverError>;
}

#[macro_export]
macro_rules! transaction {
    ($conn:ident : $id:ident $(::Connection)? => $b:block) => {{
        let mut $conn = <$id::Connection as hextacy::Atomic>::start_transaction($conn).await?;
        match $b {
            Ok(v) => match <$id::Connection as hextacy::Atomic>::commit_transaction($conn).await {
                Ok(_) => Ok(v),
                Err(e) => Err(e),
            },
            Err(e) => match <$id::Connection as hextacy::Atomic>::abort_transaction($conn).await {
                Ok(_) => Err(e),
                Err(er) => Err(er),
            },
        }
    }};
}

/// The error returned by driver implementations. Use [DriverError::Custom] if you are implementing
/// the [Driver] trait on your own types.
#[derive(Debug, Error)]
pub enum DriverError {
    #[cfg(feature = "db-mongo")]
    #[error("Mongo driver error: {0}")]
    Mongo(#[from] mongodb::error::Error),

    #[cfg(any(
        feature = "db-postgres-diesel",
        feature = "db-mysql-diesel",
        feature = "db-sqlite-diesel",
    ))]
    #[error("Postgres pool error: {0}")]
    DieselConnection(#[from] diesel::r2d2::PoolError),

    #[cfg(any(
        feature = "db-postgres-diesel",
        feature = "db-mysql-diesel",
        feature = "db-sqlite-diesel",
    ))]
    #[error("Diesel error: {0}")]
    DieselResult(#[from] diesel::result::Error),

    #[cfg(any(
        feature = "db-postgres-seaorm",
        feature = "db-mysql-seaorm",
        feature = "db-sqlite-seaorm"
    ))]
    #[error("SeaORM Error: {0}")]
    SeaormConnection(#[from] sea_orm::DbErr),

    #[cfg(feature = "cache-redis")]
    #[error("Redis pool error: {0}")]
    RedisConnection(#[from] deadpool_redis::PoolError),

    #[error("Custom driver error: {0}")]
    Custom(Box<dyn std::error::Error + Send>),
}
