use crate::ClientError;
use std::sync::Arc;

#[cfg(feature = "mongo")]
pub mod mongo;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "redis")]
pub mod redis;

#[derive(Debug)]
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

/// Trait used by clients for establishing database connections.
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
