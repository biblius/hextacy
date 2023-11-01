use crate::driver::{Atomic, Driver, DriverError};
use async_trait::async_trait;
use cfg_if::cfg_if;
use diesel::{
    connection::TransactionManager,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};

cfg_if!(
    if #[cfg(feature = "db-postgres-diesel")] {
        pub type Connection = diesel::PgConnection;
    } else if #[cfg(feature = "db-mysql-diesel")] {
        pub type Connection = diesel::MysqlConnection;
    } else if #[cfg(feature = "db-sqlite-diesel")] {
        pub type Connection = diesel::SqliteConnection;
    } else {
        compile_error! {"At least one diesel driver must be selected"}
    }
);

pub type DieselConnection = PooledConnection<ConnectionManager<Connection>>;
pub type DieselPool = Pool<ConnectionManager<Connection>>;

#[async_trait]
impl Driver for DieselPool {
    type Connection = DieselConnection;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        self.get().map_err(DriverError::DieselConnection)
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
