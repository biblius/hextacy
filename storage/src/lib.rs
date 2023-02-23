use std::cell::RefMut;

use adapters::AdapterError;
use alx_clients::db::postgres::PgPoolConnection;
use diesel::{
    connection::{AnsiTransactionManager, TransactionManager},
    PgConnection,
};

pub mod adapters;
pub mod models;
pub mod repository;

#[macro_export]
macro_rules! atomic {
    ($meth:expr, $conn:expr, $($param:expr),*) => {
        {
            use std::borrow::BorrowMut;
            match $conn {
                AtomicConn::New(mut conn) => $meth(&mut conn, $($param),*).map_err(Error::new),
                AtomicConn::Existing(mut conn) => $meth(conn.borrow_mut(), $($param),*).map_err(Error::new),
            }
        }
    };
}

pub trait AtomicRepoAccess<Conn> {
    fn connect(&self) -> Result<AtomicConn<Conn>, adapters::AdapterError>;
}

pub trait RepoAccess<Conn> {
    fn connect(&self) -> Result<Conn, adapters::AdapterError>;
}

pub enum AtomicConn<'a, T> {
    New(T),
    Existing(RefMut<'a, T>),
}

pub enum PgConnectionType {
    Pool(PgPoolConnection),
    Direct(PgConnection),
}

pub trait Atomic {
    fn start_transaction(&mut self) -> Result<(), AdapterError>;

    fn rollback_transaction(&mut self) -> Result<(), AdapterError>;

    fn commit_transaction(&mut self) -> Result<(), AdapterError>;
}

pub struct Transaction<T> {
    session: T,
}

impl<T> Transaction<T> {
    pub fn inner(&mut self) -> &mut T {
        &mut self.session
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
