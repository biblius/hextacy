use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Redis error {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Serde error {0}")]
    Serde(#[from] serde_json::Error),
}
