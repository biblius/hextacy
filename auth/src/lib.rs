pub mod csrf;
pub mod user;

use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid OTP")]
    InvalidOTP,
}

impl AuthenticationError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::InvalidOTP => StatusCode::UNAUTHORIZED,
        }
    }
}
