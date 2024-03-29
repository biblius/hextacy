use crate::core::auth::AuthenticationError;
use crate::db::adapters::AdapterError;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use deadpool_redis::redis;
use hextacy::queue::QueueError;
use serde::Serialize;
use serde_json::json;
use thiserror::Error;
use validify::ValidationErrors;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthenticationError),

    #[error("UUID: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("Crypto: {0}")]
    Crypto(#[from] hextacy::crypto::CryptoError),

    #[error("Adapter: {0}")]
    Adapter(#[from] AdapterError),

    #[error("Redis: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Validation: {0}")]
    Validation(#[from] ValidationErrors),

    #[error("Serde: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Http response: {0}")]
    HttpResponse(#[from] hextacy::web::xhttp::response::ResponseError),

    #[error("Axum response: {0}")]
    AxumResponse(#[from] axum::http::Error),

    #[error("Queue: {0}")]
    Queue(QueueError),
}

impl From<QueueError> for Error {
    fn from(value: QueueError) -> Self {
        Self::Queue(value)
    }
}

impl Error {
    pub fn new<E: Into<Self>>(e: E) -> Self {
        e.into()
    }

    /// Returns error message and description
    pub fn message_and_description(&self) -> (&'static str, String) {
        match self {
            Self::Validation(_) => ("Validation", "Invalid request parameters".to_string()),
            _ => ("Internal Server Error", "Internal server error".to_string()),
        }
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            e => {
                dbg!(e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let (message, description) = self.message_and_description();
        let error_response = match self {
            Self::Validation(errs) => {
                ErrorResponse::new(status.as_u16(), message, &description, Some(errs))
            }
            _ => ErrorResponse::new(status.as_u16(), message, &description, None),
        };

        let body = Json(json! {error_response});
        (status, body).into_response()
    }
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse<'a, T> {
    code: u16,
    message: &'a str,
    description: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<T>,
}

impl<'a, T> ErrorResponse<'a, T>
where
    T: Serialize,
{
    pub fn new(code: u16, message: &'a str, description: &'a str, details: Option<T>) -> Self {
        Self {
            code,
            message,
            description,
            details,
        }
    }
}

impl<T> std::fmt::Display for ErrorResponse<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Message: {}, Description: {}",
            self.message, self.description
        )
    }
}
