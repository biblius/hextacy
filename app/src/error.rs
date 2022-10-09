use super::models::auth::authentication::AuthenticationError;
use actix_web::{body::BoxBody, HttpResponse, HttpResponseBuilder as Response, ResponseError};
use reqwest::StatusCode;
use serde::Serialize;
use std::fmt::Display;
use thiserror::{self, Error};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Authentication Error: {0}")]
    Authentication(#[from] AuthenticationError),
}

impl Error {
    /// Returns error description
    pub fn message(&self) -> String {
        match self {
            Self::Authentication(e) => match e {
                AuthenticationError::InvalidCredentials => "Invalid credentails".to_string(),
                AuthenticationError::InvalidOTP => "Invalid one time password provided".to_string(),
                AuthenticationError::InvalidRole => "Invalid role".to_string(),
                AuthenticationError::InvalidToken => {
                    "Session token either missing or expired".to_string()
                }
            },
        }
    }

    /// Generates an http response with the given error
    pub fn respond(error: Error) -> HttpResponse {
        let status = error.status_code();
        let error_response =
            ErrorResponse::new(status.as_u16(), error.to_string(), error.message());
        Response::new(status).json(error_response)
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
        let error_response = ErrorResponse::new(status.as_u16(), self.to_string(), self.message());
        Response::new(status).json(error_response)
    }
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}

impl ErrorResponse {
    pub fn new(code: u16, error: String, message: String) -> Self {
        Self {
            code,
            error,
            message,
        }
    }
}

impl Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}
