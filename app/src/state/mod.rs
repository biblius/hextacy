pub mod reqwest_client;

use std::sync::Arc;

use infrastructure::storage::{
    postgres::{Pg, PgPoolConnection},
    redis::{Rd, RedisPoolConnection},
};

use crate::error::Error;

#[derive(Clone)]
pub struct State {
    pub client: Arc<reqwest::Client>,
    pub pg_pool: Arc<Pg>,
    pub rd_pool: Arc<Rd>,
}

impl State {
    pub fn init() -> Self {
        let client = Arc::new(reqwest_client::init());
        let pg_pool = Arc::new(Pg::new());
        let rd_pool = Arc::new(Rd::new());

        Self {
            client,
            pg_pool,
            rd_pool,
        }
    }

    pub fn pg_connect(&self) -> Result<PgPoolConnection, Error> {
        self.pg_pool.connect().map_err(|e| e.into())
    }
    pub fn rd_connect(&self) -> Result<RedisPoolConnection, Error> {
        self.rd_pool.connect().map_err(|e| e.into())
    }
}
