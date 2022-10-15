use actix_web::{body::BoxBody, HttpResponse, HttpResponseBuilder as Response, ResponseError};
use infrastructure::storage::redis;
use reqwest::StatusCode;
use serde::Serialize;
use std::fmt::Display;
use thiserror::{self, Error};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Authentication Error: {0}")]
    Authentication(#[from] AuthenticationError),
    #[error("Database Error: {0}")]
    Database(#[from] infrastructure::storage::DatabaseError),
    #[error("Diesel Error: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("Redis Error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Bcrypt Error: {0}")]
    Bcrypt(#[from] infrastructure::crypto::CryptoError),
    #[error("Serde Error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl Error {
    /// Returns error description
    pub fn message_and_description(&self) -> (&'static str, &'static str) {
        match self {
            Self::Authentication(e) => match e {
                AuthenticationError::InvalidCredentials => {
                    ("INVALID_CREDENTIALS", "Invalid credentials")
                }
                AuthenticationError::InvalidOTP => ("INVALID_2FA", "Invalid 2FA code"),
                AuthenticationError::InvalidRole => ("INVALID_ROLE", "Role does not exist"),
                AuthenticationError::InvalidToken => {
                    ("INVALID_TOKEN", "Invalid registration token")
                }
                AuthenticationError::UnverifiedEmail => ("UNVERIFIED_EMAIL", "Email not verified"),
                AuthenticationError::AccountFrozen => {
                    ("ACCOUNT_FROZEN", "Account has been suspended")
                }
            },
            _ => ("INTERNAL_SERVER_ERROR", "Internal server error"),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            Error::Authentication(e) => e.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let status = self.status_code();
        let (message, description) = self.message_and_description();
        let error_response = ErrorResponse::new(status.as_u16(), description, message);
        Response::new(status).json(error_response)
    }
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse<'a> {
    code: u16,
    message: &'a str,
    description: &'a str,
}

impl<'a> ErrorResponse<'a> {
    pub fn new(code: u16, description: &'a str, message: &'a str) -> Self {
        Self {
            code,
            message,
            description,
        }
    }
}

impl Display for ErrorResponse<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Message: {}, Description: {}",
            self.message, self.description
        )
    }
}

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid OTP")]
    InvalidOTP,
    #[error("Invalid role")]
    InvalidRole,
    #[error("Unverified email")]
    UnverifiedEmail,
    #[error("Account frozen")]
    AccountFrozen,
}

impl AuthenticationError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::InvalidOTP => StatusCode::UNAUTHORIZED,
            Self::InvalidRole => StatusCode::UNPROCESSABLE_ENTITY,
            Self::UnverifiedEmail => StatusCode::UNAUTHORIZED,
            Self::AccountFrozen => StatusCode::UNAUTHORIZED,
        }
    }
}
