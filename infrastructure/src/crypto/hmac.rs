use crate::config::env;
use data_encoding::Encoding;
use hmac::Mac;
use sha2::Sha256;
use tracing::debug;

use super::CryptoError;

// Generate an HMAC token with the given environment secret and the provided buffer.
///
/// The token is encoded to the provided encoding.
///
/// Panics if the provided `env_key` is not set in the .env file.
pub fn generate_hmac(
    env_key: &str,
    nonce: &str,
    encoding: Encoding,
) -> Result<String, CryptoError> {
    debug!("Generating HMAC with key {}", env_key);

    let hmac_secret =
        env::get(env_key).unwrap_or_else(|_| panic!("No value found for key '{env_key}'"));

    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(hmac_secret.as_bytes())?;

    hmac::Mac::update(&mut mac, nonce.as_bytes());

    Ok(encoding.encode(&mac.finalize().into_bytes()))
}

/// Verifies the given HMAC with the given nonce. Returns `Ok(true)` if the resulting hashes match, `Ok(false)` otherwise.
pub fn verify_hmac(
    env_key: &str,
    nonce: &str,
    hmac: &str,
    encoding: Encoding,
) -> Result<bool, CryptoError> {
    debug!("Verifying HMAC with key {}", env_key);
    let hmac_secret =
        env::get(env_key).unwrap_or_else(|_| panic!("No value found for key '{env_key}'"));

    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(hmac_secret.as_bytes())?;
    hmac::Mac::update(&mut mac, nonce.as_bytes());

    let original = encoding.decode(hmac.as_bytes())?;

    mac.verify_slice(&original[..])
        .map_or_else(|_| Ok(false), |_| Ok(true))
}

#[cfg(test)]
mod tests {
    use data_encoding::{BASE32, BASE64, BASE64URL};

    use crate::config;

    use super::*;

    #[test]
    fn hmac() {
        config::env::set(
            "REG_TOKEN_SECRET",
            "0e7cfad46e31c2bfd76bb0687385b87536898b209a9aef13e94b430d7d3585f7",
        );
        let nonce = uuid::Uuid::new_v4().to_string();
        let hmac = generate_hmac("REG_TOKEN_SECRET", &nonce, BASE64).unwrap();
        let res = verify_hmac("REG_TOKEN_SECRET", &nonce, &hmac, BASE64).unwrap();
        assert!(res);

        let nonce = uuid::Uuid::new_v4().to_string();
        let hmac = generate_hmac("REG_TOKEN_SECRET", &nonce, BASE32).unwrap();
        assert!(matches!(
            verify_hmac("REG_TOKEN_SECRET", &nonce, &hmac, BASE32),
            Ok(res) if res
        ));

        let nonce = uuid::Uuid::new_v4().to_string();
        let hmac = generate_hmac("REG_TOKEN_SECRET", &nonce, BASE64URL).unwrap();
        assert!(matches!(
            verify_hmac("REG_TOKEN_SECRET", &nonce, &hmac, BASE64URL),
            Ok(res) if res
        ))
    }
}
