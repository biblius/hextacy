use crate::driver::{Driver, DriverError};
use async_trait::async_trait;
use deadpool_redis::redis::{AsyncCommands, FromRedisValue, RedisError, ToRedisArgs};
use deadpool_redis::{Connection, Pool};
use serde::{de::DeserializeOwned, Serialize};

pub type RedisConnection = Connection;

#[async_trait]
impl Driver for Pool {
    type Connection = RedisConnection;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        self.get().await.map_err(DriverError::RedisConnection)
    }
}

/// Utility trait for adapters that use Redis. Provides a basic set of functionality out of the box.
#[async_trait]
pub trait RedisAdapterExt {
    type Error: From<RedisError> + From<serde_json::Error>;

    async fn get<K, V>(conn: &mut RedisConnection, key: K) -> Result<V, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
        V: FromRedisValue + Send + Sync,
    {
        let result = conn.get::<K, V>(key).await?;
        Ok(result)
    }

    /// Returns a simple string reply according to Redis' SET\[EX] command.
    /// The underlying Redis library still uses the SETEX command which is deprecated
    /// so the return value could be changed to an `Option<String>` to reflect the lib
    /// if/when it updates.
    async fn set<K, V>(
        conn: &mut RedisConnection,
        key: K,
        val: V,
        ex: Option<usize>,
    ) -> Result<String, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        if let Some(ex) = ex {
            conn.set_ex::<K, V, String>(key, val, ex)
                .await
                .map_err(Self::Error::from)
        } else {
            conn.set::<K, V, String>(key, val)
                .await
                .map_err(Self::Error::from)
        }
    }

    async fn delete<K>(conn: &mut RedisConnection, key: K) -> Result<(), Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
    {
        conn.del::<K, ()>(key).await.map_err(Self::Error::from)
    }

    async fn get_json<K, V>(conn: &mut RedisConnection, key: K) -> Result<V, Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
        V: DeserializeOwned,
    {
        let result = conn.get::<K, String>(key).await?;
        serde_json::from_str::<V>(&result).map_err(Self::Error::from)
    }

    async fn set_json<K, V>(
        conn: &mut RedisConnection,
        key: K,
        val: V,
        ex: Option<usize>,
    ) -> Result<(), Self::Error>
    where
        K: ToRedisArgs + Send + Sync,
        V: Serialize + Send + Sync,
    {
        let value = serde_json::to_string(&val)?;
        if let Some(ex) = ex {
            conn.set_ex::<K, String, ()>(key, value, ex)
                .await
                .map_err(Self::Error::from)
        } else {
            conn.set::<K, String, ()>(key, value)
                .await
                .map_err(Self::Error::from)
        }
    }
}
