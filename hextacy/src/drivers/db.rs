use crate::drivers::DriverError;
use async_trait::async_trait;
use std::sync::Arc;

#[cfg(any(feature = "db", feature = "full", feature = "mongo"))]
pub mod mongo;
#[cfg(any(
    feature = "db",
    feature = "full",
    feature = "postgres-diesel",
    feature = "postgres-seaorm"
))]
pub mod postgres;

#[derive(Debug)]
/// A struct that contains a generic driver `A` that, through [DBConnect], establishes a database connection `C`.
/// Serves as a wrapper around connections so they can stay generic and consistent while building repositories.
///
/// Service adapters utilise this for establishing connections with a uniform API. One may implement this manually or
/// use the macros provided in `hextacy_derive` for quick implementations. The derive macros generate this struct
/// internally.
pub struct Driver<A, C>
where
    A: DBConnect<Connection = C>,
{
    pub driver: Arc<A>,
}

impl<A, C> Clone for Driver<A, C>
where
    A: DBConnect<Connection = C>,
{
    fn clone(&self) -> Self {
        Self {
            driver: self.driver.clone(),
        }
    }
}

impl<A, C> Driver<A, C>
where
    A: DBConnect<Connection = C>,
{
    pub fn new(driver: Arc<A>) -> Self {
        Self { driver }
    }
}

#[async_trait]
/// Trait used by drivers for establishing database connections. The [Driver] implements this and delegates
/// the `connect` method to any concrete type that gets instantiated in it.
pub trait DBConnect {
    type Connection;
    async fn connect(&self) -> Result<Self::Connection, DriverError>;
}

#[async_trait]
impl<A, C> DBConnect for Driver<A, C>
where
    A: DBConnect<Connection = C> + Send + Sync,
{
    type Connection = C;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        self.driver.connect().await
    }
}
