pub mod hmac;
pub mod jwt;
pub mod otp;

use bcrypt;
pub use bcrypt::BcryptError;
use data_encoding::{Encoding, BASE64URL_NOPAD};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use thiserror::Error;
use tracing::debug;

pub fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[inline]
pub fn bcrypt_hash(password: &str) -> Result<String, CryptoError> {
    bcrypt::hash(password, 10).map_err(Into::into)
}

#[inline]
pub fn bcrypt_verify(password: &str, hash: &str) -> Result<bool, CryptoError> {
    bcrypt::verify(password, hash).map_err(Into::into)
}

#[inline]
pub fn pw_and_hash() -> Result<(String, String), CryptoError> {
    let pw = token(BASE64URL_NOPAD, 64);
    let hashed = bcrypt_hash(&pw)?;
    Ok((pw, hashed))
}

/// Generate a random 128 byte hmac encoded to the provided encoding.
pub fn token(encoding: Encoding, length: usize) -> String {
    debug!("Generating random token");
    let mut rng = StdRng::from_entropy();
    let mut buff = vec![0_u8; length];
    rng.fill_bytes(&mut buff);
    encoding.encode(&buff)
}

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
    HmacLength(#[from] ::hmac::digest::InvalidLength),
    #[error("{0}")]
    Hmac(#[from] ::hmac::digest::MacError),
    #[error("{0}")]
    DataEncoding(#[from] data_encoding::DecodeError),
    #[error("{0}")]
    Thotp(#[from] thotp::ThotpError),
    #[error("{0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
}
