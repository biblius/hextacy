use crate::cache::{CacheAdapterError, KeyPrefix};
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait BasicCacheAccess<C> {
    async fn get_string(
        conn: &mut C,
        id: impl KeyPrefix + Send,
        key: &str,
    ) -> Result<String, CacheAdapterError>;

    async fn get_i64(
        conn: &mut C,
        id: impl KeyPrefix + Send,
        key: &str,
    ) -> Result<i64, CacheAdapterError>;

    async fn set_str(
        conn: &mut C,
        id: impl KeyPrefix + Send,
        key: &str,
        value: &str,
        ex: Option<usize>,
    ) -> Result<(), CacheAdapterError>;

    async fn set_i64(
        conn: &mut C,
        id: impl KeyPrefix + Send,
        key: &str,
        value: i64,
        ex: Option<usize>,
    ) -> Result<(), CacheAdapterError>;

    async fn delete(
        conn: &mut C,
        id: impl KeyPrefix + Send,
        key: &str,
    ) -> Result<(), CacheAdapterError>;

    async fn get_json<T>(
        conn: &mut C,
        id: impl KeyPrefix + Send,
        key: &str,
    ) -> Result<T, CacheAdapterError>
    where
        T: DeserializeOwned;

    async fn set_json<T>(
        conn: &mut C,
        id: impl KeyPrefix + Send,
        key: &str,
        val: T,
        ex: Option<usize>,
    ) -> Result<(), CacheAdapterError>
    where
        T: Serialize + Send + Sync;

    async fn refresh(
        conn: &mut C,
        id: impl KeyPrefix + Send,
        key: &str,
        duration: i64,
    ) -> Result<(), CacheAdapterError>;

    async fn set_or_increment(
        conn: &mut C,
        id: impl KeyPrefix + Send,
        key: &str,
        amount: i64,
        ex: Option<usize>,
    ) -> Result<i64, CacheAdapterError>;
}
