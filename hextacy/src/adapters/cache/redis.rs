use crate::driver::Driver;
use deadpool_redis::redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use deadpool_redis::{Connection, Pool};
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

pub type RedisConnection = Connection;

impl Driver for Pool {
    type Connection = RedisConnection;
    type Error = deadpool_redis::PoolError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        self.get().await
    }
}

/// Utility trait for adapters that use Redis. Provides a basic set of functionality out of the box.
pub trait RedisExt {
    type Error: From<deadpool_redis::redis::RedisError> + From<serde_json::Error>;

    fn get<K, V>(
        conn: &mut RedisConnection,
        key: K,
    ) -> impl Future<Output = Result<V, Self::Error>> + Send
    where
        K: ToRedisArgs + Send + Sync,
        V: FromRedisValue + Send + Sync,
    {
        async {
            let result = conn.get::<K, V>(key).await?;
            Ok(result)
        }
    }

    /// Returns a simple string reply according to Redis' SET\[EX] command.
    /// The underlying Redis library still uses the SETEX command which is deprecated
    /// so the return value could be changed to an `Option<String>` to reflect the lib
    /// if/when it updates.
    ///
    /// `ex` is an optional expiration time in seconds.
    fn set<K, V>(
        conn: &mut RedisConnection,
        key: &K,
        val: &V,
        ex: Option<usize>,
    ) -> impl Future<Output = Result<String, Self::Error>> + Send
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        async move {
            if let Some(ex) = ex {
                conn.set_ex::<&K, &V, String>(key, val, ex)
                    .await
                    .map_err(Self::Error::from)
            } else {
                conn.set::<&K, &V, String>(key, val)
                    .await
                    .map_err(Self::Error::from)
            }
        }
    }

    fn delete<K>(
        conn: &mut RedisConnection,
        key: K,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        K: ToRedisArgs + Send + Sync,
    {
        async { conn.del::<K, ()>(key).await.map_err(Self::Error::from) }
    }

    fn get_json<K, V>(
        conn: &mut RedisConnection,
        key: K,
    ) -> impl Future<Output = Result<V, Self::Error>> + Send
    where
        K: ToRedisArgs + Send + Sync,
        V: DeserializeOwned,
    {
        async {
            let result = conn.get::<K, String>(key).await?;
            serde_json::from_str::<V>(&result).map_err(Self::Error::from)
        }
    }

    fn set_json<K, V>(
        conn: &mut RedisConnection,
        key: &K,
        val: &V,
        ex: Option<usize>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send
    where
        K: ToRedisArgs + Send + Sync,
        V: Serialize + Send + Sync,
    {
        async move {
            let value = serde_json::to_string(val)?;
            if let Some(ex) = ex {
                conn.set_ex::<&K, String, ()>(key, value, ex)
                    .await
                    .map_err(Self::Error::from)
            } else {
                conn.set::<&K, String, ()>(key, value)
                    .await
                    .map_err(Self::Error::from)
            }
        }
    }
}
