pub mod postgres;
use thiserror::Error;

use self::postgres::PgAdapterError;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Postgres Adapter Error {0}")]
    Postgres(#[from] PgAdapterError),
    #[error("Does not exist: {0}")]
    DoesNotExist(String),
}
