use std::future::Future;

/// Drivers are intended to provide a simple interface for establishing generic connections that other components
/// can use to remain decoupled from a concrete implementation. By utilising this trait, concrete data sources and clients
/// can easily be changed without interfering with the business layer.
/// A concrete implementation of a driver is usually a thin wrapper around a connection pool or a client.
///
/// The necessity for this trait arises from the fact that data sources have to pass around connections for
/// transactions. Since transactions could span multiple tables in the same execution flow, several repositories
/// must be able work on the same transaction. This is why repositories must take in a connection - the driver trait is here
/// to provide it. Services that utilise this trait can be bound to multiple repositories and one driver to provide the
/// connections for those repositories.
///
/// Check out the [adapters module][crate::adapters] to see concrete implementations.
pub trait Driver {
    type Connection;
    type Error;

    fn connect(&self) -> impl Future<Output = Result<Self::Connection, Self::Error>>;
}

/// Used for creating bounds on generic connections when the adapter needs to have atomic repository access.
///
/// This trait is used to normalise the API for transactions that are connection based and transactions that
/// return a transaction struct.
///
/// When transactions are connection based, the `TransactionResult` is typically
/// the connection on which the transaction is started.
///
/// When they are struct based, the adapter must implement a repository trait for both the
/// connection and transaction (usually a trait is provided for both so one can use it to
/// mitigate 2 different implementations).
pub trait Atomic: Sized {
    type TransactionResult;
    type Error;

    fn start_transaction(
        self,
    ) -> impl Future<Output = Result<Self::TransactionResult, Self::Error>> + Send;
    fn commit_transaction(
        tx: Self::TransactionResult,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
    fn abort_transaction(
        tx: Self::TransactionResult,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

/// Utility for grouping actions together in a transaction.
///
/// Takes in a closure and exposes a connection to it with a started transaction.
/// The closure must return a result. The closure is then matched and the transaction
/// is either commited or aborted depending on the result.
///
/// ### Example
///
/// The connection must implement [Atomic] in order to use it with this macro.
///
/// ```ignore
/// let conn = self.driver.connect().await?;
///
/// // Must be named the same as the connection variable `conn`
/// transaction!(
///     conn: Connection => {
///         insert_something(&conn, /* ... */).await?;
///         insert_something_else(&conn, /* ... */).await?;
///     }
/// )
/// ```
///
/// If any of the above create actions fail, none of them will leave any side effects.
#[macro_export]
macro_rules! transaction {
    ($conn:ident : $id:ident => $b:block) => {{
        let mut $conn = <$id as hextacy::Atomic>::start_transaction($conn).await?;
        match $b {
            Ok(v) => match <$id as hextacy::Atomic>::commit_transaction($conn).await {
                Ok(_) => Ok(v),
                Err(e) => Err(e),
            },
            Err(e) => match <$id as hextacy::Atomic>::abort_transaction($conn).await {
                Ok(_) => Err(e),
                Err(er) => Err(er),
            },
        }
    }};
}
