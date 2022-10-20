pub mod jwt;
pub mod utility;

pub use bcrypt::BcryptError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("{0}")]
    IO(#[from] std::io::Error),
    #[error("{0}")]
    JWT(#[from] jsonwebtoken::errors::Error),
    #[error("{0}")]
    Serde(#[from] serde_json::Error),
    #[error("{0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("{0}")]
    HmacLength(#[from] hmac::digest::InvalidLength),
    #[error("{0}")]
    Hmac(#[from] hmac::digest::MacError),
    #[error("{0}")]
    DataEncoding(#[from] data_encoding::DecodeError),
    #[error("{0}")]
    Thotp(#[from] thotp::ThotpError),
    #[error("{0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
}
