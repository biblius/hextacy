//! Common crypto functionalities used in web apps. Can be utilised to reduce the amount of imports.

pub mod hmac;
pub mod jwt;
pub mod otp;

use bcrypt;
pub use bcrypt::BcryptError;
use data_encoding::{Encoding, BASE64URL_NOPAD};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use thiserror::Error;
use tracing::debug;
use uuid::Uuid;

/// Generates a UUID v4.
#[inline]
pub fn uuid() -> Uuid {
    uuid::Uuid::new_v4()
}

/// Hashes the given string using the provided cost.
#[inline]
pub fn bcrypt_hash(password: &str, cost: u32) -> Result<String, CryptoError> {
    bcrypt::hash(password, cost).map_err(Into::into)
}

/// Verifies whether the given password matches the hash.
#[inline]
pub fn bcrypt_verify(password: &str, hash: &str) -> Result<bool, CryptoError> {
    bcrypt::verify(password, hash).map_err(Into::into)
}

/// Creates a password with the given length and hashes it using Bcrypt with the given cost.
/// Returns the original generated password as the first element and the hashed one as the second.
#[inline]
pub fn pw_and_hash(length: usize, cost: u32) -> Result<(String, String), CryptoError> {
    let pw = token(BASE64URL_NOPAD, length);
    let hashed = bcrypt_hash(&pw, cost)?;
    Ok((pw, hashed))
}

/// Generate an HMAC with the given length and encoded to the given encoding.
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
    #[error("{0}")]
    Env(#[from] std::env::VarError),
}
