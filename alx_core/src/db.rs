pub mod pg;

use std::cell::{RefCell, RefMut};
use thiserror::Error;

#[macro_export]
/// Macro intended as an ergonomic shortcut to get the underlying connection of [AtomicConn] enums and perform database
/// operations on it.
///
/// The first argument is always the operation to perform (a function call),
/// the second is the [AtomicConn] returned from [AtomicRepoAccess]' connect
/// method and the rest are any number of parameters that match the signature of the first argument.
///
/// ### Example
///
/// ```ignore
/// fn get_user_by_id(&self, id: &str) -> Result<User, Error> {
///     // Connect via AtomicRepoAccess implementation
///     let conn = self.connect()?;
///
///     // Perform the operation using the connection
///     atomic!(User::get_by_id, conn, id).map_err(Error::new)
/// }
/// ```
///
/// Desugars to:
///
/// ```ignore
/// fn get_user_by_id(&self, id: &str) -> Result<User, Error> {
///     // Connect via AtomicRepoAccess implementation
///     let conn = self.connect()?;
///
///     match conn {
///         AtomicConn::New(mut conn) => User::get_by_id(&mut conn, id),
///         AtomicConn::Existing(mut conn) => User::get_by_id(conn.borrow_mut().as_mut().unwrap(), id),
///     }
/// }
/// ```
macro_rules! atomic {
    ($meth:expr, $conn:expr, $($param:expr),*) => {
        {
            use std::borrow::BorrowMut;
            match $conn {
                alx_core::db::AtomicConn::New(mut conn) => $meth(&mut conn, $($param),*),
                alx_core::db::AtomicConn::Existing(mut conn) => $meth(conn.borrow_mut().as_mut().unwrap(), $($param),*),
            }
        }
    };
}

#[macro_export]
/// Takes in `self` and the closure and wraps it in a transaction. The code block must return
/// a `Result<T, E>`.
///
/// The service repository (via the `Atomic` impl) will call
/// `start_transaction()`, execute the block and based on the returned result will either commit or rollback
/// the transaction.
macro_rules! transaction {
    ($sel:expr, $b:block) => {{
        $sel.repo.start_transaction()?;
        match $b {
            Ok(res) => {
                $sel.repo.commit_transaction()?;
                Ok(res)
            }
            Err(e) => {
                $sel.repo.rollback_transaction()?;
                Err(e)
            }
        }
    }};
}

/// For use by atomic repository implementations. Represents a connection wrapped in a
/// `RefCell` that gets mutably accessed in an ongoing transaction.
pub type Transaction<C> = RefCell<Option<C>>;

/// Used for establishing connections to a database.
pub trait RepoAccess<C> {
    fn connect(&self) -> Result<C, DatabaseError>;
}

/// Used for creating transactions in repositories. Structs implementing this trait
/// should have a client and transaction field on them for establishing and storing transactions,
/// respectively.
pub trait AtomicRepoAccess<C>: Atomic {
    fn connect(&self) -> Result<AtomicConn<C>, DatabaseError>;
}

/// Represents a newly established connection or an already existing one. Intended to be used by
/// [AtomicRepoAccess] implementations. For a quick way to get the underlying connection of this enum,
/// use the [atomic!] macro.
pub enum AtomicConn<'a, T> {
    New(T),
    Existing(RefMut<'a, Option<T>>),
}

/// Used by repositories that need atomic DB access
pub trait Atomic {
    fn start_transaction(&self) -> Result<(), DatabaseError>;

    fn rollback_transaction(&self) -> Result<(), DatabaseError>;

    fn commit_transaction(&self) -> Result<(), DatabaseError>;
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Error while attempting to establish connection: {0}")]
    Client(#[from] super::clients::ClientError),
    #[error("Transaction Error: {0}")]
    Transaction(#[from] TransactionError),
    #[error("Diesel Error: {0}")]
    Diesel(#[from] diesel::result::Error),
}

#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("Cannot start new transaction while an existing on is in progress")]
    InProgress,
    #[error("Cannot rollback or commit non existing transaction")]
    NonExisting,
}
