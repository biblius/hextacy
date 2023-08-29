use super::CryptoError;
use data_encoding::Encoding;
use hmac::Mac;
use sha2::Sha256;

// Generate an HMAC token with the given secret and the provided buffer.
///
/// The token is encoded to the provided encoding.
pub fn generate_hmac(
    secret: &[u8],
    nonce: &[u8],
    encoding: Encoding,
) -> Result<String, CryptoError> {
    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(secret)?;

    hmac::Mac::update(&mut mac, nonce);

    Ok(encoding.encode(&mac.finalize().into_bytes()))
}

/// Verifies the given HMAC with the given nonce. Returns `Ok(true)` if the resulting hashes match, `Ok(false)` otherwise.
pub fn verify_hmac(
    secret: &[u8],
    nonce: &[u8],
    hmac: &[u8],
    encoding: Encoding,
) -> Result<bool, CryptoError> {
    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(secret)?;

    hmac::Mac::update(&mut mac, nonce);

    let original = encoding.decode(hmac)?;

    mac.verify_slice(&original[..])
        .map_or_else(|_| Ok(false), |_| Ok(true))
}

#[cfg(test)]
mod tests {
    use data_encoding::{BASE32, BASE64, BASE64URL};

    use super::*;

    #[test]
    fn hmac() {
        let secret = "0e7cfad46e31c2bfd76bb0687385b87536898b209a9aef13e94b430d7d3585f7";
        let nonce = uuid::Uuid::new_v4();
        let hmac = generate_hmac(secret.as_bytes(), nonce.as_bytes(), BASE64).unwrap();
        let res =
            verify_hmac(secret.as_bytes(), nonce.as_bytes(), hmac.as_bytes(), BASE64).unwrap();
        assert!(res);

        let nonce = uuid::Uuid::new_v4();
        let hmac = generate_hmac(secret.as_bytes(), nonce.as_bytes(), BASE32).unwrap();
        assert!(matches!(
            verify_hmac(secret.as_bytes(), nonce.as_bytes(), hmac.as_bytes(), BASE32),
            Ok(res) if res
        ));

        let nonce = uuid::Uuid::new_v4();
        let hmac = generate_hmac(secret.as_bytes(), nonce.as_bytes(), BASE64URL).unwrap();
        assert!(matches!(
            verify_hmac(secret.as_bytes(), nonce.as_bytes(), hmac.as_bytes(), BASE64URL),
            Ok(res) if res
        ))
    }
}
