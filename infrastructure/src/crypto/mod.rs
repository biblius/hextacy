pub mod bin;
pub mod jwt;
pub mod rsa_key_pair;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    JWT(#[from] jsonwebtoken::errors::Error),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
}
