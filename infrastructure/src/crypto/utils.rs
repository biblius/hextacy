use super::CryptoError;
use crate::config::env;
use bcrypt;
use data_encoding::BASE64URL;
use hmac::{self, Mac};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use sha2::Sha256;
use std::fmt::Write;

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
pub fn generate_hmac(env_key: &str, nonce: Option<&str>) -> Result<String, CryptoError> {
    let hmac_secret =
        env::get(env_key).unwrap_or_else(|_| panic!("No value found for key '{}'", env_key));

    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(hmac_secret.as_bytes())?;

    let nonce = if let Some(nonce) = nonce {
        nonce.to_string()
    } else {
        random_nonce()
    };

    hmac::Mac::update(&mut mac, nonce.as_bytes());

    Ok(BASE64URL.encode(&mac.finalize().into_bytes()))
}

/// Verifies the given HMAC with the given nonce. Returns `Ok(true)` if the resulting hashes match, `Ok(false)` otherwise.
pub fn verify_hmac(env_key: &str, nonce: &str, hmac: &str) -> Result<bool, CryptoError> {
    let hmac_secret =
        env::get(env_key).unwrap_or_else(|_| panic!("No value found for key '{}'", env_key));

    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(hmac_secret.as_bytes())?;
    hmac::Mac::update(&mut mac, nonce.as_bytes());

    let original = BASE64URL.decode(hmac.as_bytes())?;

    mac.verify_slice(&original)
        .map_or_else(|_| Ok(false), |_| Ok(true))
}

pub fn verify_otp(password: &str, secret: &str) -> Result<(bool, i16), CryptoError> {
    let secret = secret.as_bytes();
    thotp::verify_totp(password, secret, 0).map_err(|e| e.into())
}

/// Generates a random nonce, useful for caching temporary tokens
fn random_nonce() -> String {
    let mut buff = [0_u8; 256];
    let mut rng = StdRng::from_entropy();
    rng.fill_bytes(&mut buff);
    to_hex(&buff)
}

/// Utility for encoding a buffer to a hex string
fn to_hex(buf: &[u8]) -> String {
    let mut r = String::new();
    for b in buf {
        write!(r, "{:02x}", b).unwrap();
    }
    r
}

#[cfg(test)]
mod tests {
    use crate::config;

    use super::*;

    #[test]
    fn hmac() {
        config::env::set(
            "CSRF_SECRET",
            "0e7cfad46e31c2bfd76bb0687385b87536898b209a9aef13e94b430d7d3585f7",
        );
        let nonce = uuid::Uuid::new_v4().to_string();
        let hmac = generate_hmac("CSRF_SECRET", Some(&nonce)).unwrap();
        assert!(matches!(verify_hmac("CSRF_SECRET", &hmac, &nonce), Ok(_)))
    }
}
