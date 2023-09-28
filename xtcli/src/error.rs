use thiserror::Error;

#[derive(Debug, Error)]
pub enum AlxError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}
