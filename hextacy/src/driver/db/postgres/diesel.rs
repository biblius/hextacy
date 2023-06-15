use crate::db::DatabaseError;
use crate::driver::DriverError;
use crate::{db::Atomic, driver::Driver};
use async_trait::async_trait;
use diesel::{
    connection::TransactionManager,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};

pub type DieselConnection = PooledConnection<ConnectionManager<PgConnection>>;

/// Contains a connection pool for postgres with diesel. An instance of this
/// should be shared through the app with Arcs
#[derive(Debug, Clone)]
pub struct PostgresDiesel {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl PostgresDiesel {
    pub fn new(
        host: &str,
        port: u16,
        user: &str,
        password: &str,
        db: &str,
        pool_size: u32,
    ) -> Self {
        let url = format!("postgresql://{user}:{password}@{host}:{port}/{db}");

        let manager = ConnectionManager::<PgConnection>::new(url);

        let pool = Pool::builder()
            .max_size(pool_size)
            .build(manager)
            .unwrap_or_else(|e| panic!("Failed to create postgres pool: {e}"));

        tracing::debug!(
            "Successfully initialised PG pool (diesel) at {}",
            format!("postgressql://{user}:***@{host}:{port}/{db}")
        );

        Self { pool }
    }
}

#[async_trait]
impl Driver for PostgresDiesel {
    type Connection = DieselConnection;

    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        match self.pool.get() {
            Ok(conn) => Ok(conn),
            Err(e) => Err(DriverError::DieselConnection(e.to_string())),
        }
    }
}

#[async_trait]
impl Atomic for DieselConnection {
    type TransactionResult = Self;
    async fn start_transaction(mut self) -> Result<Self, DatabaseError> {
        diesel::connection::AnsiTransactionManager::begin_transaction(&mut *self)?;
        Ok(self)
    }

    async fn commit_transaction(mut tx: Self::TransactionResult) -> Result<(), DatabaseError> {
        diesel::connection::AnsiTransactionManager::commit_transaction(&mut *tx)?;
        Ok(())
    }

    async fn abort_transaction(mut tx: Self::TransactionResult) -> Result<(), DatabaseError> {
        diesel::connection::AnsiTransactionManager::rollback_transaction(&mut *tx)?;
        Ok(())
    }
}
