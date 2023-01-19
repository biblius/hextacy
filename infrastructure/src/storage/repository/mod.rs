pub mod role;
pub mod session;
pub mod user;

use crate::{clients::ClientError, storage::adapters::postgres::PgAdapterError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("{0}")]
    Adapter(#[from] super::adapters::AdapterError),
    #[error("{0}")]
    PgAdapter(#[from] PgAdapterError),
    #[error("Connection Error: {0}")]
    Client(#[from] ClientError),
    #[error("Diesel Error: {0}")]
    Diesel(#[from] diesel::result::Error),
}
