use crate::driver::{Atomic, Driver, DriverError};
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use sea_orm::TransactionTrait;

#[cfg(all(
    not(feature = "db-postgres-seaorm"),
    not(feature = "db-mysql-seaorm"),
    not(feature = "db-sqlite-seaorm")
))]
compile_error! {"At least one seaorm driver must be selected"}

/// Driver connectin used by sea_orm
pub use sea_orm::DatabaseConnection;

#[async_trait]
impl Driver for DatabaseConnection {
    type Connection = Self;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        // Internally sea-orm uses sqlx whose pool struct contains an arc
        // that gets cloned via this
        Ok(self.clone())
    }
}

#[async_trait]
impl Atomic for DatabaseConnection {
    type TransactionResult = DatabaseTransaction;

    async fn start_transaction(mut self) -> Result<Self::TransactionResult, DriverError> {
        DatabaseConnection::begin(&self)
            .await
            .map_err(DriverError::SeaormConnection)
    }

    async fn commit_transaction(tx: Self::TransactionResult) -> Result<(), DriverError> {
        DatabaseTransaction::commit(tx)
            .await
            .map_err(DriverError::SeaormConnection)
    }

    async fn abort_transaction(tx: Self::TransactionResult) -> Result<(), DriverError> {
        DatabaseTransaction::rollback(tx)
            .await
            .map_err(DriverError::SeaormConnection)
    }
}
