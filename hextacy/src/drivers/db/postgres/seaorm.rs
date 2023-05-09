use crate::{
    db::{Atomic, DatabaseError},
    drivers::{db::Connect, DriverError},
};
use async_trait::async_trait;
use sea_orm::TransactionTrait;
use sea_orm::{ConnectOptions, Database, DatabaseTransaction};

pub use sea_orm::DatabaseConnection;

/// Contains a connection pool for postgres with sea-orm. An instance of this
/// should be shared through the app with Arcs
#[derive(Debug, Clone)]
pub struct PostgresSea {
    pool: DatabaseConnection,
}

impl PostgresSea {
    pub async fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        db: &str,
        pool_size: u32,
    ) -> Self {
        let url = format!("postgresql://{user}:{password}@{host}:{port}/{db}");

        let mut options = ConnectOptions::new(url);

        options.max_connections(pool_size);

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
impl Connect for PostgresSea {
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

    async fn start_transaction(mut self) -> Result<Self::TransactionResult, DatabaseError> {
        let tx = DatabaseConnection::begin(&self).await?;
        Ok(tx)
    }

    async fn commit_transaction(tx: Self::TransactionResult) -> Result<(), DatabaseError> {
        DatabaseTransaction::commit(tx).await?;
        Ok(())
    }

    async fn abort_transaction(tx: Self::TransactionResult) -> Result<(), DatabaseError> {
        DatabaseTransaction::rollback(tx).await?;
        Ok(())
    }
}
