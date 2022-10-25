use super::CryptoError;
use bcrypt;
use data_encoding::{Encoding, BASE64URL_NOPAD};
use rand::{rngs::StdRng, RngCore, SeedableRng};
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
    let pw = token(BASE64URL_NOPAD, 64)?;
    let hashed = bcrypt_hash(&pw)?;
    Ok((pw, hashed))
}

/// Generate a random 128 byte hmac encoded to the provided encoding.
pub fn token(encoding: Encoding, length: usize) -> Result<String, CryptoError> {
    debug!("Generating random token");
    let mut rng = StdRng::from_entropy();
    let mut buff = vec![0_u8; length];
    rng.fill_bytes(&mut buff);
    Ok(encoding.encode(&buff))
}
