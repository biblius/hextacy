use crate::driver::{Atomic, Driver, DriverError};
use async_trait::async_trait;
use mongodb::{
    options::{ClientOptions, Credential, ServerAddress},
    Client, ClientSession,
};
use tracing::trace;

pub use mongodb;

/// Thin wrapper around a [mongodb::Client] that implements [Driver] that can be
/// injected into services.
#[derive(Debug, Clone)]
pub struct Mongo {
    pub driver: Client,
}

impl Mongo {
    pub fn new(host: &str, port: u16, user: &str, password: &str, db: &str) -> Self {
        let address = ServerAddress::Tcp {
            host: host.to_string(),
            port: Some(port),
        };

        let credential = Credential::builder()
            .password(password.to_string())
            .username(user.to_string())
            .build();

        let options = ClientOptions::builder()
            .hosts(vec![address])
            .credential(credential)
            .default_database(db.to_string())
            .build();

        let client = match Client::with_options(options) {
            Ok(driver) => {
                trace!("Built Mongo driver");
                Self { driver }
            }
            Err(e) => panic!("Error occurred while building Mongo driver: {e}"),
        };

        tracing::debug!(
            "Successfully initialised Mongo client at {}",
            format!("mongodb://{user}:***@{host}:{port}/{db}")
        );

        client
    }
}

#[async_trait]
impl Driver for Mongo {
    type Connection = ClientSession;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        self.driver
            .start_session(None)
            .await
            .map_err(DriverError::Mongo)
    }
}

#[async_trait]
impl Atomic for ClientSession {
    type TransactionResult = Self;

    async fn start_transaction(mut self) -> Result<Self, DriverError> {
        ClientSession::start_transaction(&mut self, None).await?;
        Ok(self)
    }

    async fn commit_transaction(mut tx: Self::TransactionResult) -> Result<(), DriverError> {
        ClientSession::commit_transaction(&mut tx).await?;
        Ok(())
    }

    async fn abort_transaction(mut tx: Self::TransactionResult) -> Result<(), DriverError> {
        ClientSession::abort_transaction(&mut tx).await?;
        Ok(())
    }
}
