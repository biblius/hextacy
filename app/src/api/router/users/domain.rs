use super::{
    contract::{RepositoryContract, ServiceContract},
    data::{GetUsersPaginated, UserResponse},
};
use crate::error::Error;
use actix_web::HttpResponse;
use async_trait::async_trait;
use infrastructure::utility::http::response::Response;
use reqwest::StatusCode;

pub(super) struct UserService<R: RepositoryContract> {
    pub repository: R,
}

#[async_trait]
impl<R> ServiceContract for UserService<R>
where
    R: RepositoryContract + Send + Sync,
{
    async fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error> {
        let users = self
            .repository
            .get_paginated(
                data.page.unwrap_or(1_u16),
                data.per_page.unwrap_or(25),
                data.sort_by,
            )
            .await?;

        Ok(UserResponse::new(users).to_response(StatusCode::OK, None, None))
    }
}
