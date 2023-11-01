use crate::driver::{Atomic, Driver, DriverError};
use async_trait::async_trait;
use mongodb::{Client, ClientSession};

#[async_trait]
impl Driver for Client {
    type Connection = ClientSession;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        self.start_session(None).await.map_err(DriverError::Mongo)
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
