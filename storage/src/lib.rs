use adapters::AdapterError;
use diesel::{
    connection::{AnsiTransactionManager, TransactionManager},
    PgConnection,
};

pub mod adapters;
pub mod models;
pub mod repository;

pub trait PgRepository {
    fn connect(&self) -> Result<PgConnection, adapters::AdapterError>;

    fn transaction(&self) -> Result<Transaction<PgConnection>, AdapterError> {
        let mut conn = self.connect()?;
        <AnsiTransactionManager as TransactionManager<PgConnection>>::begin_transaction(&mut conn)
            .map_err(AdapterError::from)?;
        Ok(Transaction { trx: conn })
    }
}

pub struct Transaction<T> {
    trx: T,
}

impl<T> Transaction<T> {
    pub fn inner(&mut self) -> &mut T {
        &mut self.trx
    }
}

impl Transaction<PgConnection> {
    pub fn commit(&mut self) -> Result<(), AdapterError> {
        <AnsiTransactionManager as TransactionManager<PgConnection>>::commit_transaction(
            self.inner(),
        )
        .map_err(AdapterError::from)
    }

    pub fn rollback(&mut self) -> Result<(), AdapterError> {
        <AnsiTransactionManager as TransactionManager<PgConnection>>::rollback_transaction(
            self.inner(),
        )
        .map_err(AdapterError::from)
    }
}

pub trait DieselTransaction {
    fn transaction(&mut self) -> &mut PgConnection;
}

impl DieselTransaction for Transaction<PgConnection> {
    fn transaction(&mut self) -> &mut PgConnection {
        self.inner()
    }
}
