use super::{data::GetUsersPaginated, infrastructure::Postgres, response::UserResponse};
use crate::error::Error;
use actix_web::HttpResponse;
use infrastructure::{http::response::Response, storage::postgres::Pg};
use reqwest::StatusCode;
use std::sync::Arc;

pub(super) struct Users {
    database: Postgres,
}
impl Users {
    pub(super) async fn get_paginated(
        &self,
        data: GetUsersPaginated,
    ) -> Result<HttpResponse, Error> {
        let (total, result) = self.database.get_paginated(
            data.page.unwrap_or(1 as u16),
            data.per_page.unwrap_or(25),
            data.sort_by,
        )?;

        Ok(UserResponse::new(total as usize, result).to_response(StatusCode::OK, None, None))
    }

    pub(super) fn new(pg: Arc<Pg>) -> Self {
        Self {
            database: Postgres::new(pg),
        }
    }
}
