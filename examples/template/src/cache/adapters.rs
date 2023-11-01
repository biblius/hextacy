use super::contracts::BasicCacheAccess;
use crate::error::Error;
use async_trait::async_trait;
use chrono::Utc;
use deadpool_redis::redis::AsyncCommands;
use hextacy::adapters::cache::redis::{RedisAdapterExt, RedisConnection};
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone)]
pub struct RedisAdapter;

impl RedisAdapter {
    fn key(key: &str) -> String {
        format!("auth:{key}")
    }
}

impl RedisAdapterExt for RedisAdapter {
    type Error = Error;
}

#[async_trait]
impl BasicCacheAccess<RedisConnection> for RedisAdapter {
    async fn set_str(
        &self,
        conn: &mut RedisConnection,
        key: &str,
        value: &str,
        ex: Option<usize>,
    ) -> Result<(), Error> {
        let key = Self::key(key);
        <Self as RedisAdapterExt>::set(conn, key, value, ex).await?;
        Ok(())
    }

    async fn get_str(&self, conn: &mut RedisConnection, key: &str) -> Result<String, Error> {
        let key = Self::key(key);
        <Self as RedisAdapterExt>::get(conn, key)
            .await
            .map_err(Error::new)
    }

    async fn set_i64(
        &self,
        conn: &mut RedisConnection,
        key: &str,
        value: i64,
        ex: Option<usize>,
    ) -> Result<(), Error> {
        let key = Self::key(key);
        Self::set(conn, key, value, ex).await.map_err(Error::new)?;
        Ok(())
    }

    async fn get_i64(&self, conn: &mut RedisConnection, key: &str) -> Result<i64, Error> {
        let key = Self::key(key);
        <Self as RedisAdapterExt>::get(conn, key)
            .await
            .map_err(Error::new)
    }

    async fn get_json<T>(&self, conn: &mut RedisConnection, key: &str) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let key = Self::key(key);
        <Self as RedisAdapterExt>::get_json(conn, key)
            .await
            .map_err(Error::new)
    }

    async fn set_json<T>(
        &self,
        conn: &mut RedisConnection,

        key: &str,
        val: T,
        ex: Option<usize>,
    ) -> Result<(), Error>
    where
        T: Serialize + Send + Sync,
    {
        let key = Self::key(key);
        <Self as RedisAdapterExt>::set_json(conn, key, val, ex)
            .await
            .map_err(Error::new)
    }

    async fn delete(&self, conn: &mut RedisConnection, key: &str) -> Result<(), Error> {
        let key = Self::key(key);
        <Self as RedisAdapterExt>::delete(conn, key)
            .await
            .map_err(Error::new)
    }

    async fn refresh(
        &self,
        conn: &mut RedisConnection,

        key: &str,
        duration: i64,
    ) -> Result<(), Error> {
        let key = Self::key(key);
        conn.expire_at(key, (Utc::now().timestamp() + duration % i64::MAX) as usize)
            .await
            .map_err(Error::new)
    }
}
