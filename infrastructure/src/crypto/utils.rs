use super::CryptoError;
use crate::config::env;
use bcrypt;
use data_encoding::BASE64URL;
use hmac::{self, Mac};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use sha2::Sha256;

#[inline]
pub fn bcrypt_hash(password: &str) -> Result<String, CryptoError> {
    bcrypt::hash(password, 10).map_err(|e| e.into())
}

#[inline]
pub fn bcrypt_verify(password: &str, hash: &str) -> Result<bool, CryptoError> {
    bcrypt::verify(password, hash).map_err(|e| e.into())
}

/// Generate an HMAC token with the given environment secret and the provided buffer.
///
/// Panics if the provided `env_key` is not set in the .env file.
pub fn generate_hmac(env_key: &str, buffer: &str) -> Result<String, CryptoError> {
    let hmac_secret = env::get(env_key).expect(&format!("No value found for key '{}'", env_key));
    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(hmac_secret.as_bytes())?;
    hmac::Mac::update(&mut mac, &buffer.as_bytes());
    Ok(BASE64URL.encode(&mac.finalize().into_bytes()))
}

/// Generate an HMAC token with an environment secret and a random buffer.
///
/// Panics if the provided `env_key` is not set in the .env file.
pub fn generate_hmac_random(env_key: &str) -> Result<String, CryptoError> {
    let mut buff = [0_u8; 256];
    let mut rng = StdRng::from_entropy();
    rng.fill_bytes(&mut buff);
    let hmac_secret = env::get(env_key).expect(&format!("No value found for key '{}'", env_key));
    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(hmac_secret.as_bytes())?;
    hmac::Mac::update(&mut mac, &buff);
    Ok(BASE64URL.encode(&mac.finalize().into_bytes()))
}

pub fn verify_otp(password: &str, secret: &str) -> Result<(bool, i16), CryptoError> {
    let secret = secret.as_bytes();
    thotp::verify_totp(password, secret, 0).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn hmac() {
        /* let (hmac, nonce) = generate_hmac("CSRF_SECRET").unwrap();
        assert!(matches!(verify_hmac("CSRF_SECRET", &hmac, &nonce), Ok(_))) */
    }
}
