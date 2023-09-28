use crate::driver::{Atomic, Driver, DriverError};
use async_trait::async_trait;
use cfg_if::cfg_if;
use diesel::{
    connection::TransactionManager,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

cfg_if!(
    if #[cfg(feature = "db-postgres-diesel")] {
        type Connection = diesel::PgConnection;
    } else if #[cfg(feature = "db-mysql-diesel")] {
        type Connection = diesel::MysqlConnection;
    } else if #[cfg(feature = "db-sqlite-diesel")] {
        type Connection = diesel::SqliteConnection;
    } else {
        compile_error!("At least one diesel driver must be selected")
    }
);

pub type DieselConnection = PooledConnection<ConnectionManager<Connection>>;

/// Thin wrapper around a diesel postgres connection pool.
#[derive(Debug, Clone)]
pub struct DieselDriver {
    pool: Pool<ConnectionManager<Connection>>,
}

impl DieselDriver {
    pub fn new(url: &str) -> Self {
        let pool = Pool::builder()
            .build(ConnectionManager::<Connection>::new(url))
            .expect("Could not establish database connection");
        Self { pool }
    }
}

#[async_trait]
impl Driver for DieselDriver {
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
