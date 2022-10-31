use thiserror::Error;

#[derive(Debug, Error)]
pub enum AlxError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serde: {0}")]
    Serde(#[from] serde_yaml::Error),
}
