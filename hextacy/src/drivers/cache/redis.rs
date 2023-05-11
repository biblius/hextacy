use std::fmt::{Debug, Display};

use async_trait::async_trait;
pub use redis;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    cache::{CacheAccess, CacheAccessJson, CacheError},
    drivers::{Connect, DriverError},
};
use deadpool_redis::{Config, Connection, Pool, Runtime};
use r2d2::PooledConnection;
use redis::{AsyncCommands, ConnectionInfo, FromRedisValue, IntoConnectionInfo, ToRedisArgs};

pub type RedisConnection = PooledConnection<redis::Client>;

/// Contains a redis connection pool. An instance of this should be shared through the app with arcs.
#[derive(Clone)]
pub struct Redis {
    pool: Pool,
}

impl Redis {
    pub fn new(
        host: &str,
        port: u16,
        user: Option<&str>,
        password: Option<&str>,
        db: i64,
        max_size: usize,
    ) -> Self {
        let conn_info = connection_info(host, port, user, password, db);
        let pool = Config::from_connection_info(conn_info)
            .builder()
            .expect("Could not create redis pool builder")
            .max_size(max_size)
            .runtime(Runtime::Tokio1)
            .build()
            .expect("Could not create redis connection pool");
        Self { pool }
    }
}

fn connection_info(
    host: &str,
    port: u16,
    user: Option<&str>,
    password: Option<&str>,
    db: i64,
) -> ConnectionInfo {
    let db_url = format!("redis://{host}:{port}");
    let mut conn_info = db_url.into_connection_info().unwrap();
    conn_info.redis.password = password.map(|pw| pw.to_string());
    conn_info.redis.username = user.map(|uname| uname.to_string());
    conn_info.redis.db = db;
    conn_info
}

#[async_trait]
impl Connect for Redis {
    type Connection = Connection;
    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        match self.pool.get().await {
            Ok(conn) => Ok(conn),
            Err(e) => Err(DriverError::RedisConnection(e.to_string())),
        }
    }
}

#[async_trait]
impl<K, V> CacheAccess<Connection, K, V> for Redis
where
    K: ToRedisArgs + Send + Sync,
    V: ToRedisArgs + FromRedisValue + Send + Sync,
{
    async fn get(&self, key: &K) -> Result<V, CacheError> {
        let mut conn = self.connect().await?;
        let result = conn.get::<&K, V>(key).await?;
        drop(conn);
        Ok(result)
    }

    async fn set(&self, key: &K, val: &V, ex: Option<usize>) -> Result<(), CacheError> {
        let mut conn = self.connect().await?;
        if let Some(ex) = ex {
            conn.set_ex::<&K, &V, ()>(key, val, ex)
                .await
                .map_err(Into::into)
        } else {
            conn.set::<&K, &V, ()>(key, val).await.map_err(Into::into)
        }
    }

    async fn delete(&self, key: &K) -> Result<(), CacheError> {
        let mut conn = self.connect().await?;
        conn.del::<&K, ()>(key).await.map_err(Into::into)
    }
}

#[async_trait]
impl<K, V> CacheAccessJson<Connection, K, V> for Redis
where
    K: ToRedisArgs + Send + Sync,
    V: Serialize + DeserializeOwned + Send + Sync,
{
    async fn get_json(&self, key: &K) -> Result<V, CacheError> {
        let mut conn = self.connect().await?;
        let result = conn.get::<&K, String>(key).await?;
        serde_json::from_str::<V>(&result).map_err(Into::into)
    }

    async fn set_json(&self, key: &K, val: &V, ex: Option<usize>) -> Result<(), CacheError> {
        let mut conn = self.connect().await?;
        let value = serde_json::to_string(val)?;
        if let Some(ex) = ex {
            conn.set_ex::<&K, String, ()>(key, value, ex)
                .await
                .map_err(Into::into)
        } else {
            conn.set::<&K, String, ()>(key, value)
                .await
                .map_err(Into::into)
        }
    }
}

impl Debug for Redis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Redis").field("pool", &"{ ... }").finish()
    }
}
