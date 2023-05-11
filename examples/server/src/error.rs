use actix_web::{body::BoxBody, HttpResponse, HttpResponseBuilder as Response, ResponseError};
use hextacy::drivers::cache::redis;
use reqwest::StatusCode;
use serde::Serialize;
use std::fmt::Display;
use thiserror::{self, Error};
use validify::{ValidationError, ValidationErrors};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Authentication Error: {0}")]
    Authentication(#[from] AuthenticationError),
    #[error("Env var Error: {0}")]
    Var(#[from] std::env::VarError),
    #[error("Driver Error: {0}")]
    Driver(#[from] hextacy::drivers::DriverError),
    #[error("Database Error: {0}")]
    Database(#[from] hextacy::db::DatabaseError),
    #[error("Cache Error: {0}")]
    Cache(#[from] hextacy::cache::CacheError),
    #[error("Adapter Error: {0}")]
    Adapter(#[from] crate::db::adapters::AdapterError),
    #[error("Redis Error: {0}")]
    Redis(#[from] redis::redis::RedisError),
    #[error("Crypto Error: {0}")]
    Crypto(#[from] hextacy::crypto::CryptoError),
    #[error("Diesel Error: {0}")]
    Diesel(#[from] diesel::result::Error),
    #[error("Serde Error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Reqwest Header Error: {0}")]
    HeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error("Reqwest Header Error: {0}")]
    ToStr(#[from] reqwest::header::ToStrError),
    #[error("Validation Error")]
    Validation(Vec<ValidationError>),
    #[error("Http Error")]
    Http(#[from] hextacy::web::http::HttpError),
    #[error("OAuth Provider Error")]
    OAuthProvider(#[from] crate::services::oauth::OAuthProviderError),
    /// Useful for testing when you need an error response
    #[error("None")]
    #[allow(dead_code)]
    None,
}

impl From<ValidationErrors> for Error {
    fn from(e: ValidationErrors) -> Self {
        Self::Validation(e.errors().to_vec())
    }
}

impl Error {
    pub fn new<E: Into<Self>>(e: E) -> Self {
        e.into()
    }

    /// Returns error message and description
    pub fn message_and_description(&self) -> (&'static str, String) {
        match self {
            Self::Authentication(e) => e.describe(),
            Self::Adapter(crate::db::adapters::AdapterError::DoesNotExist) => {
                ("NOT_FOUND", "Resource does not exist".to_string())
            }
            Self::Validation(_) => ("VALIDATION", "Invalid input".to_string()),
            _ => ("INTERNAL_SERVER_ERROR", "Internal server error".to_string()),
        }
    }

    /// Check whether the error is a validation error
    fn check_validation_errors(&self) -> Option<Vec<ValidationError>> {
        match self {
            Error::Validation(errors) => Some(errors.clone()),
            _ => None,
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

    /// Transform the error to an `ErrorResponse` struct that implements actix's `ErrorResponse` trait.
    /// Flattens all validation errors to a vec, if any
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let status = self.status_code();
        let (message, description) = self.message_and_description();
        let validation_errors = self.check_validation_errors();
        let error_response =
            ErrorResponse::new(status.as_u16(), message, &description, validation_errors);
        Response::new(status).json(error_response)
    }
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse<'a> {
    code: u16,
    message: &'a str,
    description: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    validation_errors: Option<Vec<ValidationError>>,
}

impl<'a> ErrorResponse<'a> {
    pub fn new(
        code: u16,
        message: &'a str,
        description: &'a str,
        validation_errors: Option<Vec<ValidationError>>,
    ) -> Self {
        Self {
            code,
            message,
            description,
            validation_errors,
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
    Unauthenticated,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken(&'static str),
    #[error("Invalid OTP")]
    InvalidOTP,
    #[error("Invalid CSRF header")]
    InvalidCsrfHeader,
    #[error("Insufficient rights")]
    InsufficientRights,
    #[error("Account frozen")]
    AccountFrozen,
    #[error("Unverified email")]
    EmailUnverified,
    #[error("Authentication blocked")]
    AuthBlocked,
}

impl AuthenticationError {
    pub fn status_code(&self) -> StatusCode {
        use self::AuthenticationError::*;
        match self {
            Unauthenticated => StatusCode::UNAUTHORIZED,
            InvalidCredentials => StatusCode::UNAUTHORIZED,
            InvalidToken(_) => StatusCode::UNAUTHORIZED,
            InsufficientRights => StatusCode::FORBIDDEN,
            InvalidOTP => StatusCode::UNAUTHORIZED,
            InvalidCsrfHeader => StatusCode::UNAUTHORIZED,
            AccountFrozen => StatusCode::UNAUTHORIZED,
            EmailUnverified => StatusCode::UNAUTHORIZED,
            AuthBlocked => StatusCode::UNAUTHORIZED,
        }
    }

    pub fn describe(&self) -> (&'static str, String) {
        match self {
            AuthenticationError::Unauthenticated => ("UNAUTHORIZED", "No session".to_string()),
            AuthenticationError::InvalidCsrfHeader => {
                ("UNAUTHORIZED", "Invalid CSRF header".to_string())
            }
            AuthenticationError::InvalidCredentials => {
                ("UNAUTHORIZED", "Invalid credentials".to_string())
            }
            AuthenticationError::InvalidOTP => ("UNAUTHORIZED", "Invalid OTP provided".to_string()),
            AuthenticationError::AccountFrozen => ("SUSPENDED", "Account suspended".to_string()),
            AuthenticationError::EmailUnverified => {
                ("UNVERIFIED", "Email not verified".to_string())
            }
            AuthenticationError::AuthBlocked => {
                ("BLOCKED", "Authentication currently blocked".to_string())
            }
            AuthenticationError::InsufficientRights => {
                ("FORBIDDEN", "Insufficient rights".to_string())
            }
            AuthenticationError::InvalidToken(id) => match *id {
                "OTP" => ("INVALID_TOKEN", "Invalid OTP token".to_string()),
                "Registration" => ("INVALID_TOKEN", "Invalid registration token".to_string()),
                "Password" => ("INVALID_TOKEN", "Invalid password change token".to_string()),
                "OAuth" => ("INVALID_TOKEN", "Invalid OAuth access token".to_string()),
                _ => ("INVALID_TOKEN", "Token not found".to_string()),
            },
        }
    }
}
