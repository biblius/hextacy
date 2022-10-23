pub mod cookie;
pub mod cors;
pub mod request;
pub mod response;
pub mod security_headers;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("Request error: {0}")]
    Request(String),
}
