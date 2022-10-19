use std::sync::Arc;

use super::response::UserResponse;
use crate::{error::Error, models::user::User};
use actix_web::HttpResponse;
use infrastructure::{http::response::Response, storage::postgres::Pg};
use reqwest::StatusCode;

pub struct Users {
    pg: Postgres,
}
impl Users {
    pub async fn get_all(&self) -> Result<HttpResponse, Error> {
        Ok(UserResponse::new(self.pg.get_all()).to_response(StatusCode::OK, None, None))
    }

    pub(crate) fn new(pg: Arc<Pg>) -> Self {
        Self {
            pg: Postgres::new(pg),
        }
    }
}

struct Postgres {
    pool: Arc<Pg>,
}

impl Postgres {
    fn new(pool: Arc<Pg>) -> Self {
        Self { pool }
    }

    fn get_all(&self) -> Vec<User> {
        User::get_all(&mut self.pool.connect().unwrap()).unwrap()
    }
}
