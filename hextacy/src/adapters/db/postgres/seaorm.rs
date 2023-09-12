use crate::driver::{Atomic, Driver, DriverError};
use async_trait::async_trait;
use sea_orm::TransactionTrait;
use sea_orm::{ConnectOptions, Database, DatabaseTransaction};

/// Driver connectin used by sea_orm
pub use sea_orm::DatabaseConnection;

/// Contains a connection pool for postgres with sea-orm. An instance of this
/// should be shared through the app with Arcs
#[derive(Debug, Clone)]
pub struct SeaPgDriver {
    pool: DatabaseConnection,
}

impl SeaPgDriver {
    pub async fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        db: &str,
        pool_size: Option<u32>,
    ) -> Self {
        let url = format!("postgresql://{user}:{password}@{host}:{port}/{db}");

        let mut options = ConnectOptions::new(url);

        options.max_connections(pool_size.unwrap_or(8));

        let pool = Database::connect(options)
            .await
            .expect("Could not establish PostgresSea connection");

        tracing::debug!(
            "Successfully initialised PG pool (seaorm) at {}",
            format!("postgressql://{user}:***@{host}:{port}/{db}")
        );

        Self { pool }
    }
}

#[async_trait]
impl Driver for SeaPgDriver {
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
