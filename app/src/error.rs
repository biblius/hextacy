use actix_web::{body::BoxBody, HttpResponse, HttpResponseBuilder as Response, ResponseError};
use infrastructure::storage::redis::{self, CacheId};
use reqwest::StatusCode;
use serde::Serialize;
use std::fmt::Display;
use thiserror::{self, Error};
use validator::{ValidationError, ValidationErrors};

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
    #[error("Validation error detected")]
    Validation(Vec<ValidationError>),
}

impl From<ValidationErrors> for Error {
    fn from(e: ValidationErrors) -> Self {
        let mut errors = vec![];
        Self::nest_validation_errors(e, &mut errors);
        Self::Validation(errors)
    }
}

impl Error {
    pub fn new<E: Into<Self>>(e: E) -> Self {
        e.into()
    }

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
                AuthenticationError::InsufficientRights => (
                    "FORBIDDEN",
                    "You do not have the necessary rights to view this page",
                ),
                AuthenticationError::InvalidToken(id) => match id {
                    CacheId::OTPToken => ("INVALID_TOKEN", "Invalid OTP token"),
                    CacheId::RegToken => ("INVALID_TOKEN", "Invalid registration token"),
                    CacheId::PWToken => (
                        "INVALID_TOKEN",
                        "Temporary password expired. Log in to obtain a new one",
                    ),
                    _ => ("INVALID_TOKEN", "Invalid token"),
                },
                AuthenticationError::AccountFrozen => {
                    ("ACCOUNT_FROZEN", "Account has been suspended")
                }
                AuthenticationError::EmailTaken => ("EMAIL_TAKEN", "Cannot use provided email"),
            },
            Self::Validation(_) => ("VALIDATION_ERROR", "Invalid data detected"),
            _ => ("INTERNAL_SERVER_ERROR", "Internal server error"),
        }
    }

    fn check_validation_errors(&self) -> Option<Vec<ValidationError>> {
        match self {
            Error::Validation(errors) => Some(errors.clone()),
            _ => None,
        }
    }

    fn nest_validation_errors(errs: ValidationErrors, buff: &mut Vec<ValidationError>) {
        for err in errs.errors().values() {
            match err {
                validator::ValidationErrorsKind::Struct(box_error) => {
                    Self::nest_validation_errors(*box_error.clone(), buff);
                }
                validator::ValidationErrorsKind::List(e) => {
                    for er in e.clone().into_values() {
                        Self::nest_validation_errors(*er.clone(), buff);
                    }
                }
                validator::ValidationErrorsKind::Field(e) => {
                    for er in e {
                        buff.push(er.clone());
                    }
                }
            }
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

        let validation_errors = self.check_validation_errors();

        let error_response =
            ErrorResponse::new(status.as_u16(), description, message, validation_errors);
        Response::new(status).json(error_response)
    }
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse<'a> {
    code: u16,
    message: &'a str,
    description: &'a str,
    validation_errors: Option<Vec<ValidationError>>,
}

impl<'a> ErrorResponse<'a> {
    pub fn new(
        code: u16,
        description: &'a str,
        message: &'a str,
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
    SessionNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken(CacheId),
    #[error("Invalid OTP")]
    InvalidOTP,
    #[error("Invalid CSRF header")]
    InvalidCsrfHeader,
    #[error("Insufficient right")]
    InsufficientRights,
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
            Self::InsufficientRights => StatusCode::FORBIDDEN,
            Self::InvalidOTP => StatusCode::UNAUTHORIZED,
            Self::InvalidCsrfHeader => StatusCode::UNAUTHORIZED,
            Self::AccountFrozen => StatusCode::UNAUTHORIZED,
            Self::EmailTaken => StatusCode::CONFLICT,
        }
    }
}
