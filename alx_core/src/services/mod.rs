pub mod email;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("Transport error: {0}")]
    Transport(#[from] lettre::transport::smtp::Error),
    #[error("Email error: {0}")]
    Email(#[from] lettre::error::Error),
}
