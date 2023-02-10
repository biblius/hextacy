pub mod schema;
pub mod session;
pub mod user;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PgAdapterError {
    #[error("Client error {0}")]
    Client(#[from] alx_clients::ClientError),
    #[error("Diesel error {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("Does not exist: {0}")]
    DoesNotExist(String),
}

impl PgAdapterError {
    pub fn new<E: Into<Self>>(e: E) -> Self {
        e.into()
    }
}
