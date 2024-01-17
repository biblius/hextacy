use crate::driver::{Atomic, Driver};
use mongodb::{Client, ClientSession};

impl Driver for Client {
    type Connection = ClientSession;
    type Error = mongodb::error::Error;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        self.start_session(None).await
    }
}

impl Atomic for ClientSession {
    type TransactionResult = Self;
    type Error = mongodb::error::Error;

    async fn start_transaction(mut self) -> Result<Self, Self::Error> {
        ClientSession::start_transaction(&mut self, None).await?;
        Ok(self)
    }

    async fn commit_transaction(mut tx: Self::TransactionResult) -> Result<(), Self::Error> {
        ClientSession::commit_transaction(&mut tx).await?;
        Ok(())
    }

    async fn abort_transaction(mut tx: Self::TransactionResult) -> Result<(), Self::Error> {
        ClientSession::abort_transaction(&mut tx).await?;
        Ok(())
    }
}
