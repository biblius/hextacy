pub mod postgres;
use self::postgres::PgAdapterError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Postgres Adapter Error {0}")]
    Postgres(#[from] PgAdapterError),
    #[error("Does not exist: {0}")]
    DoesNotExist(String),
    #[error("Client: {0}")]
    Client(#[from] clients::ClientError),
    #[error("Diesel: {0}")]
    Diesel(#[from] diesel::result::Error),
}
