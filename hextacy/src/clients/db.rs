use crate::clients::ClientError;
use async_trait::async_trait;
use std::sync::Arc;

pub mod mongo;
pub mod postgres;
pub mod redis;

#[derive(Debug, Clone)]
/// A struct that contains a generic client `A` that through [DBConnect] establishes a database connection `C`.
/// Serves as a wrapper around connections so they can stay generic while building repositories.
///
/// Each repository must have a client to use for establishing connections. One may implement this manually or
/// use the macros provided in `hextacy_derive` for quick implementations. The derive macros generate this struct
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

#[async_trait(?Send)]
/// Trait used by clients for establishing database connections. The [Client] implements this and delegates
/// the `connect` method to any concrete type that gets instantiated in it.
pub trait DBConnect {
    type Connection;
    async fn connect(&self) -> Result<Self::Connection, ClientError>;
}

#[async_trait(?Send)]
impl<A, C> DBConnect for Client<A, C>
where
    A: DBConnect<Connection = C>,
{
    type Connection = C;

    async fn connect(&self) -> Result<Self::Connection, ClientError> {
        self.client.connect().await
    }
}
