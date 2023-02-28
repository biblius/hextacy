use crate::ClientError;
use std::sync::Arc;

#[cfg(feature = "mongo")]
pub mod mongo;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "redis")]
pub mod redis;

#[derive(Debug, Clone)]
/// A struct that contains a generic client `A` that through [DBConnect] establishes a database connection `C`.
/// Serves as a wrapper around connections so they can stay generic while building repositories.
///
/// Each repository must have a client to use for establishing connections. One may implement this manually or
/// use the macros provided in `alx_derive` for quick implementations. The derive macros generate this struct
/// internally.
pub struct Client<A, C>
where
    A: DBConnect<Connection = C>,
{
    pub client: Arc<A>,
}

impl<A, C> Client<A, C>
where
    A: DBConnect<Connection = C>,
{
    pub fn new(client: Arc<A>) -> Self {
        Self { client }
    }
}

/// Trait used by clients for establishing database connections. The [Client] implements this and delegates
/// the `connect` method to any concrete type that gets instantiated in it.
pub trait DBConnect {
    type Connection;
    fn connect(&self) -> Result<Self::Connection, ClientError>;
}

impl<A, C> DBConnect for Client<A, C>
where
    A: DBConnect<Connection = C>,
{
    type Connection = C;

    fn connect(&self) -> Result<Self::Connection, ClientError> {
        self.client.connect()
    }
}
