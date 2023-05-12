use crate::{
    cache::CacheError,
    drivers::{Connect, DriverError},
};
use async_trait::async_trait;
use deadpool_redis::{Config, Connection, Pool, Runtime};
use redis::{AsyncCommands, ConnectionInfo, FromRedisValue, IntoConnectionInfo, ToRedisArgs};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub use redis;

pub type RedisConnection = Connection;

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
        self.pool.get().await.map_err(DriverError::RedisConnection)
    }
}

#[async_trait]
pub trait RedisAdapterExt {
    async fn get<K, V>(conn: &mut RedisConnection, key: K) -> Result<V, CacheError>
    where
        K: ToRedisArgs + Send + Sync,
        V: FromRedisValue + Send + Sync,
    {
        let result = conn.get::<K, V>(key).await?;
        Ok(result)
    }

    async fn set<K, V>(
        conn: &mut RedisConnection,
        key: K,
        val: V,
        ex: Option<usize>,
    ) -> Result<(), CacheError>
    where
        K: ToRedisArgs + Send + Sync,
        V: ToRedisArgs + Send + Sync,
    {
        if let Some(ex) = ex {
            conn.set_ex::<K, V, ()>(key, val, ex)
                .await
                .map_err(CacheError::Redis)
        } else {
            conn.set::<K, V, ()>(key, val)
                .await
                .map_err(CacheError::Redis)
        }
    }

    async fn delete<K>(conn: &mut RedisConnection, key: K) -> Result<(), CacheError>
    where
        K: ToRedisArgs + Send + Sync,
    {
        conn.del::<K, ()>(key).await.map_err(CacheError::Redis)
    }

    async fn get_json<K, V>(conn: &mut RedisConnection, key: K) -> Result<V, CacheError>
    where
        K: ToRedisArgs + Send + Sync,
        V: DeserializeOwned,
    {
        let result = conn.get::<K, String>(key).await?;
        serde_json::from_str::<V>(&result).map_err(CacheError::Serde)
    }

    async fn set_json<K, V>(
        conn: &mut RedisConnection,
        key: K,
        val: V,
        ex: Option<usize>,
    ) -> Result<(), CacheError>
    where
        K: ToRedisArgs + Send + Sync,
        V: Serialize + Send + Sync,
    {
        let value = serde_json::to_string(&val)?;
        if let Some(ex) = ex {
            conn.set_ex::<K, String, ()>(key, value, ex)
                .await
                .map_err(CacheError::Redis)
        } else {
            conn.set::<K, String, ()>(key, value)
                .await
                .map_err(CacheError::Redis)
        }
    }
}

impl Debug for Redis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Redis").field("pool", &"{ ... }").finish()
    }
}
