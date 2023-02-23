pub mod postgres;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Entry does not exist")]
    DoesNotExist,
    #[error("Client: {0}")]
    Client(#[from] alx_clients::ClientError),
    #[error("Diesel: {0}")]
    Diesel(diesel::result::Error),
    #[error("Atomic Error: {0}")]
    Atomic(String),
}

impl From<diesel::result::Error> for AdapterError {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => Self::DoesNotExist,
            e => Self::Diesel(e),
        }
    }
}
