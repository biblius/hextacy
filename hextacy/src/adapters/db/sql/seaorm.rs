use crate::driver::{Atomic, Driver};
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

impl Driver for DatabaseConnection {
    type Connection = Self;
    type Error = sea_orm::DbErr;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        // Internally sea-orm uses sqlx whose pool struct contains an arc
        // that gets cloned via this
        Ok(self.clone())
    }
}

impl Atomic for DatabaseConnection {
    type TransactionResult = DatabaseTransaction;
    type Error = sea_orm::DbErr;

    async fn start_transaction(self) -> Result<Self::TransactionResult, Self::Error> {
        DatabaseConnection::begin(&self).await
    }

    async fn commit_transaction(tx: Self::TransactionResult) -> Result<(), Self::Error> {
        DatabaseTransaction::commit(tx).await
    }

    async fn abort_transaction(tx: Self::TransactionResult) -> Result<(), Self::Error> {
        DatabaseTransaction::rollback(tx).await
    }
}
