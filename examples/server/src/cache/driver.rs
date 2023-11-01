use async_trait::async_trait;
use deadpool_redis::{redis::IntoConnectionInfo, Config, Pool, Runtime};
use hextacy::{adapters::cache::redis::RedisConnection, Driver, DriverError};

/// Contains a redis deadpool instance.
#[derive(Clone)]
pub struct RedisDriver {
    pool: Pool,
}

impl std::fmt::Debug for RedisDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisDriver")
            .field("pool", &"{ ... }")
            .finish()
    }
}

impl RedisDriver {
    pub fn new(host: &str, port: u16, user: Option<&str>, password: Option<&str>, db: i64) -> Self {
        let db_url = format!("redis://{host}:{port}");
        let mut conn_info = db_url.clone().into_connection_info().unwrap();
        conn_info.redis.password = password.map(|pw| pw.to_string());
        conn_info.redis.username = user.map(|uname| uname.to_string());
        conn_info.redis.db = db;
        let pool = Config::from_connection_info(conn_info)
            .builder()
            .expect("Could not create redis pool builder")
            .runtime(Runtime::Tokio1)
            .build()
            .expect("Could not create redis connection pool");
        Self { pool }
    }
}

#[async_trait]
impl Driver for RedisDriver {
    type Connection = RedisConnection;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        self.pool.get().await.map_err(DriverError::RedisConnection)
    }
}
