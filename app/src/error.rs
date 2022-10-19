use actix_web::{body::BoxBody, HttpResponse, HttpResponseBuilder as Response, ResponseError};
use infrastructure::storage::redis::{self, CacheId};
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
    #[error("Email Error: {0}")]
    Email(#[from] infrastructure::email::EmailError),
    #[error("Reqwest Header Error: {0}")]
    Reqwest(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Reqwest Header Error: {0}")]
    ToStr(#[from] reqwest::header::ToStrError),
}

impl Error {
    /// Returns error description
    pub fn message_and_description(&self) -> (&'static str, &'static str) {
        match self {
            Self::Authentication(e) => match e {
                AuthenticationError::SessionNotFound => ("INVALID_SESSION", "Session not found"),
                AuthenticationError::InvalidCredentials => {
                    ("INVALID_CREDENTIALS", "Invalid credentials")
                }
                AuthenticationError::InvalidOTP => ("INVALID_OTP", "Invalid OTP provided"),
                AuthenticationError::InvalidCsrfHeader => (
                    "INVALID_CSRF",
                    "You do not have the rights to access this page",
                ),
                AuthenticationError::InvalidToken(id) => match id {
                    CacheId::OTPToken => ("INVALID_TOKEN", "Invalid OTP token"),
                    CacheId::RegToken => ("INVALID_TOKEN", "Invalid registration token"),
                    CacheId::PWToken => ("INVALID_TOKEN", "Invalid password token"),
                    _ => ("INVALID_TOKEN", "Invalid token"),
                },
                AuthenticationError::AccountFrozen => {
                    ("ACCOUNT_FROZEN", "Account has been suspended")
                }
                AuthenticationError::EmailTaken => ("EMAIL_TAKEN", "Cannot use provided email"),
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
    #[error("Session not found")]
    SessionNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken(CacheId),
    #[error("Invalid OTP")]
    InvalidOTP,
    #[error("Invalid CSRF header")]
    InvalidCsrfHeader,
    #[error("Account frozen")]
    AccountFrozen,
    #[error("Email taken")]
    EmailTaken,
}

impl AuthenticationError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::SessionNotFound => StatusCode::UNAUTHORIZED,
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::InvalidToken(_) => StatusCode::UNAUTHORIZED,
            Self::InvalidOTP => StatusCode::UNAUTHORIZED,
            Self::InvalidCsrfHeader => StatusCode::UNAUTHORIZED,
            Self::AccountFrozen => StatusCode::UNAUTHORIZED,
            Self::EmailTaken => StatusCode::CONFLICT,
        }
    }
}
