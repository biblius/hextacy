pub mod response;
pub mod security_headers;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Request error: {0}")]
    Request(String),
    #[error("Http error: {0}")]
    Http(#[from] actix_web::error::HttpError),
}
