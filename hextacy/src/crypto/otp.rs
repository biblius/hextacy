use super::CryptoError;
use data_encoding::Encoding;

/// Generates an OTP secret
pub fn generate_secret(size: usize, encoding: Encoding) -> String {
    thotp::encoding::encode(&thotp::generate_secret(size), encoding)
}

/// Generates a QR code svg with the given secret
pub fn generate_totp_qr_code(
    secret: &str,
    user_email: &str,
    label: &str,
    issuer: &str,
) -> Result<String, CryptoError> {
    let uri = thotp::qr::otp_uri(
        "totp",
        secret,
        &format!("{label}:{user_email}"),
        issuer,
        None,
    )?;
    thotp::qr::generate_code_svg(&uri, None, None, thotp::qr::EcLevel::M).map_err(Into::into)
}

/// Verifies a timed OTP against the given secret
pub fn verify_otp(password: &str, secret: &str, encoding: Encoding) -> Result<bool, CryptoError> {
    let secret = encoding.decode(secret.as_bytes())?;
    thotp::verify_totp(password, &secret, 0)
        .map_err(Into::into)
        .map(|(res, _)| res)
}
