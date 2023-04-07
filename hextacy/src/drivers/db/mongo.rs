use super::DBConnect;
use crate::{
    db::{Atomic, DatabaseError},
    drivers::DriverError,
};
use async_trait::async_trait;
use mongodb::{
    options::{ClientOptions, Credential, ServerAddress},
    Client, ClientSession,
};
use tracing::trace;

#[derive(Debug)]
pub struct Mongo {
    pub driver: Client,
}

impl Mongo {
    pub fn new(host: &str, port: u16, user: &str, password: &str, db: &str) -> Self {
        let options = driver_options(host, port, user, password, db);
        match Client::with_options(options) {
            Ok(driver) => {
                trace!("Built Mongo driver");
                Self { driver }
            }
            Err(e) => panic!("Error occurred while building Mongo driver: {e}"),
        }
    }

    pub fn direct(host: &str, port: u16, user: &str, password: &str, db: &str) -> Self {
        let mut options = driver_options(host, port, user, password, db);
        options.direct_connection = true.into();
        match Client::with_options(options) {
            Ok(driver) => {
                trace!("Built Mongo driver with direct connection");
                Self { driver }
            }
            Err(e) => panic!("Error occurred while building sync Mongo driver: {e}"),
        }
    }
}

fn driver_options(host: &str, port: u16, user: &str, password: &str, db: &str) -> ClientOptions {
    let address = ServerAddress::Tcp {
        host: host.to_string(),
        port: Some(port),
    };

    let credential = Credential::builder()
        .password(password.to_string())
        .username(user.to_string())
        .build();

    trace!("Building Mongo driver options with {address}");

    ClientOptions::builder()
        .hosts(vec![address])
        .credential(credential)
        .default_database(db.to_string())
        .build()
}

#[async_trait]
impl DBConnect for Mongo {
    type Connection = ClientSession;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        trace!("Mongo - Attempting pooled connection");
        let session = self.driver.start_session(None).await?;
        Ok(session)
    }
}

#[async_trait]
impl Atomic for ClientSession {
    type TransactionResult = Self;
    async fn start_transaction(mut self) -> Result<Self, DatabaseError> {
        ClientSession::start_transaction(&mut self, None).await?;
        Ok(self)
    }

    async fn commit_transaction(mut tx: Self::TransactionResult) -> Result<(), DatabaseError> {
        ClientSession::commit_transaction(&mut tx).await?;
        Ok(())
    }

    async fn abort_transaction(mut tx: Self::TransactionResult) -> Result<(), DatabaseError> {
        ClientSession::abort_transaction(&mut tx).await?;
        Ok(())
    }
}
