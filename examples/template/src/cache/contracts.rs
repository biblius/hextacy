use crate::error::Error;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

#[async_trait]
pub trait BasicCacheAccess<C> {
    async fn get_str(&self, conn: &mut C, key: &str) -> Result<String, Error>;

    async fn get_i64(&self, conn: &mut C, key: &str) -> Result<i64, Error>;

    async fn set_str(
        &self,
        conn: &mut C,
        key: &str,
        value: &str,
        ex: Option<usize>,
    ) -> Result<(), Error>;

    async fn set_i64(
        &self,
        conn: &mut C,
        key: &str,
        value: i64,
        ex: Option<usize>,
    ) -> Result<(), Error>;

    async fn delete(&self, conn: &mut C, key: &str) -> Result<(), Error>;

    async fn get_json<T>(&self, conn: &mut C, key: &str) -> Result<T, Error>
    where
        T: DeserializeOwned;

    async fn set_json<T>(
        &self,
        conn: &mut C,
        key: &str,
        val: T,
        ex: Option<usize>,
    ) -> Result<(), Error>
    where
        T: Serialize + Send + Sync;

    async fn refresh(&self, conn: &mut C, key: &str, duration: i64) -> Result<(), Error>;
}
