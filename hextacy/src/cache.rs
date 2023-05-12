use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Driver error {0}")]
    Driver(#[from] crate::drivers::DriverError),
    #[error("Redis error {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serde error {0}")]
    Serde(#[from] serde_json::Error),
}
