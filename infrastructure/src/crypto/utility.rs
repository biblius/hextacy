use super::CryptoError;
use crate::config::env;
use bcrypt;
use data_encoding::{Encoding, BASE32};
use hmac::{self, Mac};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use sha2::Sha256;
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

/// Generate a random token encoded to the provided encoding.
pub fn token(encoding: Encoding) -> Result<String, CryptoError> {
    debug!("Generating random token");

    let mut rng = StdRng::from_entropy();
    let mut buff = [0_u8; 256];
    rng.fill_bytes(&mut buff);

    let mac = hmac::Hmac::<Sha256>::new_from_slice(&buff)?;

    Ok(encoding.encode(&mac.finalize().into_bytes()))
}

/// Generate an HMAC token with the given environment secret and the provided buffer.
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
        env::get(env_key).unwrap_or_else(|_| panic!("No value found for key '{}'", env_key));

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
        env::get(env_key).unwrap_or_else(|_| panic!("No value found for key '{}'", env_key));

    let mut mac = hmac::Hmac::<Sha256>::new_from_slice(hmac_secret.as_bytes())?;
    hmac::Mac::update(&mut mac, nonce.as_bytes());

    let original = encoding.decode(hmac.as_bytes())?;

    mac.verify_slice(&original[..])
        .map_or_else(|_| Ok(false), |_| Ok(true))
}

pub fn generate_otp_secret() -> String {
    debug!("Generating OTP secret");
    thotp::encoding::encode(&thotp::generate_secret(160), BASE32)
}

pub fn generate_totp_qr_code(secret: &str, user_email: &str) -> Result<String, CryptoError> {
    debug!("Generating TOTP");
    let uri = thotp::qr::otp_uri(
        "totp",
        secret,
        &format!("RPSChat:{}", user_email),
        "RPS Chat",
        None,
    )?;
    thotp::qr::generate_code_svg(&uri, None, None, thotp::qr::EcLevel::M).map_err(Into::into)
}

pub fn verify_otp(password: &str, secret: &str) -> Result<(bool, i16), CryptoError> {
    debug!(
        "Verifying TOTP for password {} and secret {}",
        password, secret
    );
    let secret = BASE32.decode(secret.as_bytes())?;
    thotp::verify_totp(password, &secret, 0).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use data_encoding::{BASE64, BASE64URL};

    use crate::config;

    use super::*;

    #[test]
    fn hmac() {
        config::env::set(
            "CSRF_SECRET",
            "0e7cfad46e31c2bfd76bb0687385b87536898b209a9aef13e94b430d7d3585f7",
        );
        let nonce = uuid::Uuid::new_v4().to_string();
        let hmac = generate_hmac("CSRF_SECRET", &nonce, BASE64).unwrap();
        println!("{hmac}");
        let res = verify_hmac("CSRF_SECRET", &nonce, &hmac, BASE64).unwrap();
        assert!(res);

        let nonce = uuid::Uuid::new_v4().to_string();
        let hmac = generate_hmac("CSRF_SECRET", &nonce, BASE32).unwrap();
        assert!(matches!(
            verify_hmac("CSRF_SECRET", &nonce, &hmac, BASE32),
            Ok(res) if res
        ));

        let nonce = uuid::Uuid::new_v4().to_string();
        let hmac = generate_hmac("CSRF_SECRET", &nonce, BASE64URL).unwrap();
        assert!(matches!(
            verify_hmac("CSRF_SECRET", &nonce, &hmac, BASE64URL),
            Ok(res) if res
        ))
    }
}
