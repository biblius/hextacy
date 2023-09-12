use actix_web::{body::BoxBody, HttpResponse, HttpResponseBuilder as Response, ResponseError};
use hextacy::{adapters::email::TemplateMailerError, exports::deadpool_redis::redis};
use reqwest::StatusCode;
use serde::Serialize;
use std::fmt::Debug;
use thiserror::{self, Error};
use validify::{ValidationError, ValidationErrors};

use crate::{cache::TokenType, services::oauth::OAuthProviderError};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Authentication Error: {0}")]
    Authentication(#[from] AuthenticationError),
    #[error("Env var Error: {0}")]
    Var(#[from] std::env::VarError),
    #[error("Driver Error: {0}")]
    Driver(#[from] hextacy::DriverError),
    #[error("Cache Error: {0}")]
    Cache(#[from] crate::cache::CacheAdapterError),
    #[error("Adapter Error: {0}")]
    Adapter(#[from] crate::db::RepoAdapterError),
    #[error("Redis Error: {0}")]
    Redis(#[from] redis::RedisError),
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
    #[error("Http Error: {0}")]
    Http(#[from] hextacy::web::http::HttpError),
    #[error("OAuth Provider Error: {0}")]
    OAuthProvider(#[from] crate::services::oauth::OAuthProviderError),
    #[error("Smtp Mailer: {0}")]
    TemplateMailer(#[from] TemplateMailerError),
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

    /// Returns error error and description
    pub fn error_and_description(&self) -> (&'static str, &'static str) {
        match self {
            Self::Authentication(e) => e.describe(),
            Self::Adapter(crate::db::RepoAdapterError::DoesNotExist) => {
                ("NOT_FOUND", "Resource does not exist")
            }
            Self::Validation(_) => ("VALIDATION", "Invalid request parameters"),
            Self::OAuthProvider(_) => ("OAUTH", "Something went wrong"),
            _ => ("INTERNAL", "Something went wrong"),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            Self::Authentication(e) => e.status_code(),
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Adapter(crate::db::RepoAdapterError::DoesNotExist) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Transform the error to an `ErrorResponse` struct that implements actix's `ErrorResponse` trait.
    /// Flattens all validation errors to a vec, if any
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let status = self.status_code();
        let (error, description) = self.error_and_description();
        match self {
            Self::Adapter(crate::db::RepoAdapterError::DoesNotExist) => todo!(),
            Self::Validation(errs) => {
                let error_response = ErrorResponse::new_with_details(error, description, errs);
                Response::new(status).json(error_response)
            }
            Self::OAuthProvider(OAuthProviderError::GithubOAuth(err)) => {
                let error_response = ErrorResponse::new_with_details(error, description, err);
                Response::new(status).json(error_response)
            }
            _ => Response::new(status).json(ErrorResponse::<()>::new(error, description)),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse<'a, T> {
    error: &'a str,
    description: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<T>,
}

impl<'a, T> ErrorResponse<'a, T>
where
    T: Serialize,
{
    pub fn new(error: &'a str, description: &'a str) -> Self {
        Self {
            error,
            description,
            details: None,
        }
    }

    pub fn new_with_details(error: &'a str, description: &'a str, details: T) -> Self {
        Self {
            error,
            description,
            details: Some(details),
        }
    }
}

impl<T: Debug> std::fmt::Display for ErrorResponse<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Message: {}, Description: {}, Details: {:?}",
            self.error, self.description, self.details
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
    InvalidToken(TokenType),
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

    pub fn describe(&self) -> (&'static str, &'static str) {
        match self {
            AuthenticationError::Unauthenticated => ("UNAUTHORIZED", "No session"),
            AuthenticationError::InvalidCsrfHeader => ("UNAUTHORIZED", "Invalid CSRF header"),
            AuthenticationError::InvalidCredentials => ("UNAUTHORIZED", "Invalid credentials"),
            AuthenticationError::InvalidOTP => ("UNAUTHORIZED", "Invalid OTP provided"),
            AuthenticationError::AccountFrozen => ("SUSPENDED", "Account suspended"),
            AuthenticationError::EmailUnverified => ("UNVERIFIED", "Email not verified"),
            AuthenticationError::AuthBlocked => ("BLOCKED", "Authentication currently blocked"),
            AuthenticationError::InsufficientRights => ("FORBIDDEN", "Insufficient rights"),
            AuthenticationError::InvalidToken(id) => match *id {
                TokenType::OTPToken => ("INVALID_TOKEN", "Invalid OTP token"),
                TokenType::RegToken => ("INVALID_TOKEN", "Invalid registration token"),
                TokenType::PWToken => ("INVALID_TOKEN", "Invalid password change token"),
                // Token => ("INVALID_TOKEN", "Invalid OAuth access token"),
                _ => ("INVALID_TOKEN", "Token not found"),
            },
        }
    }
}
