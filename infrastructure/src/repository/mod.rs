pub mod role;
pub mod session;
pub mod user;

use crate::adapters::postgres::PgAdapterError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("{0}")]
    Adapter(#[from] PgAdapterError),
}
