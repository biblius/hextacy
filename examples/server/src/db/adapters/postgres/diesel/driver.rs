use async_trait::async_trait;
use diesel::r2d2::{ConnectionManager, Pool};
use hextacy::{Driver, DriverError};

use hextacy::adapters::db::sql::diesel::{Connection, DieselConnection, DieselPool};

/// Thin wrapper around a diesel postgres connection pool for show.
#[derive(Debug, Clone)]
pub struct DieselDriver {
    pool: DieselPool,
}

impl DieselDriver {
    pub fn new(url: &str) -> Self {
        let pool = Pool::builder()
            .build(ConnectionManager::<Connection>::new(url))
            .expect("Could not establish database connection");
        Self { pool }
    }
}

/// Just delegates to impl from hextacy.
#[async_trait]
impl Driver for DieselDriver {
    type Connection = DieselConnection;
    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        self.pool.connect().await
    }
}
