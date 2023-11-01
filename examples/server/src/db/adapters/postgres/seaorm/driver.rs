use async_trait::async_trait;
use hextacy::{Driver, DriverError};
use sea_orm::DatabaseConnection;
use sea_orm::{ConnectOptions, Database};

/// Contains a connection pool for postgres with sea-orm. An instance of this
/// should be shared through the app with Arcs
#[derive(Debug, Clone)]
pub struct SeaormDriver {
    pool: DatabaseConnection,
}

impl SeaormDriver {
    pub async fn new(url: &str) -> Self {
        let pool = Database::connect(ConnectOptions::new(url))
            .await
            .expect("Could not establish database connection");
        Self { pool }
    }
}

#[async_trait]
impl Driver for SeaormDriver {
    type Connection = DatabaseConnection;
    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        self.pool.connect().await
    }
}
