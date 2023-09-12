use crate::driver::{Atomic, Driver, DriverError};
use async_trait::async_trait;
use diesel::{
    connection::TransactionManager,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};

/// Driver connection used by diesel.
pub type DieselConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// Thin wrapper around a diesel postgres connection pool.
#[derive(Debug, Clone)]
pub struct DieselPgDriver {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DieselPgDriver {
    pub fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        db: &str,
        pool_size: Option<u32>,
    ) -> Self {
        let url = format!("postgresql://{user}:{password}@{host}:{port}/{db}");

        let manager = ConnectionManager::<PgConnection>::new(url);

        let pool = Pool::builder()
            .max_size(pool_size.unwrap_or(8))
            .build(manager)
            .unwrap_or_else(|e| panic!("Failed to create postgres pool: {e}"));

        tracing::debug!(
            "Successfully initialised PG pool (diesel) at {}",
            format!("postgressql://{user}:***@{host}:{port}/{db}")
        );

        Self { pool }
    }
}

#[async_trait]
impl Driver for DieselPgDriver {
    type Connection = DieselConnection;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        self.pool.get().map_err(DriverError::DieselConnection)
    }
}

#[async_trait]
impl Atomic for DieselConnection {
    type TransactionResult = Self;

    async fn start_transaction(mut self) -> Result<Self, DriverError> {
        diesel::connection::AnsiTransactionManager::begin_transaction(&mut *self)?;
        Ok(self)
    }

    async fn commit_transaction(mut tx: Self::TransactionResult) -> Result<(), DriverError> {
        diesel::connection::AnsiTransactionManager::commit_transaction(&mut *tx)?;
        Ok(())
    }

    async fn abort_transaction(mut tx: Self::TransactionResult) -> Result<(), DriverError> {
        diesel::connection::AnsiTransactionManager::rollback_transaction(&mut *tx)?;
        Ok(())
    }
}
