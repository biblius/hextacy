use crate::driver::{Atomic, Driver, DriverError};
use async_trait::async_trait;
use sea_orm::TransactionTrait;
use sea_orm::{ConnectOptions, Database, DatabaseTransaction};

#[cfg(all(
    not(feature = "db-postgres-seaorm"),
    not(feature = "db-mysql-seaorm"),
    not(feature = "db-sqlite-seaorm")
))]
compile_error! {"At least one seaorm driver must be selected"}

/// Driver connectin used by sea_orm
pub use sea_orm::DatabaseConnection;

/// Contains a connection pool for postgres with sea-orm. An instance of this
/// should be shared through the app with Arcs
#[derive(Debug, Clone)]
pub struct SeaormDriver {
    pool: DatabaseConnection,
}

impl SeaormDriver {
    pub async fn new(url: &str) -> Self {
        let pool = Database::connect(ConnectOptions::new(url))
            .await
            .expect("Could not establish database connection");
        Self { pool }
    }
}

#[async_trait]
impl Driver for SeaormDriver {
    type Connection = DatabaseConnection;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        // Internally sea-orm uses sqlx whose pool struct contains an arc
        // that gets cloned via this
        Ok(self.pool.clone())
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
