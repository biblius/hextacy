pub mod reqwest_client;

use infrastructure::storage::{self, postgres::PgPool, redis::RedisPool};

#[derive(Clone)]
pub struct AppState {
    pub client: reqwest::Client,
    pub pg_pool: PgPool,
    pub rd_pool: RedisPool,
}

impl AppState {
    pub fn init() -> Self {
        let client = reqwest_client::init();
        let pg_pool = storage::postgres::build_pool();
        let rd_pool = storage::redis::build_pool();

        Self {
            client,
            pg_pool,
            rd_pool,
        }
    }
}
