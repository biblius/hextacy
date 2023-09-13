use super::{
    adapters::UsersRepositoryContract,
    contract::ServiceContract,
    data::{GetUsersPaginated, UserResponse},
};
use crate::error::Error;
use async_trait::async_trait;
use hextacy::web::{http::Response, xhttp::response::RestResponse};

use reqwest::StatusCode;

pub struct Users<R>
where
    R: UsersRepositoryContract,
{
    pub repository: R,
}

#[async_trait]
impl<R> ServiceContract for Users<R>
where
    R: UsersRepositoryContract + Send + Sync,
{
    async fn get_paginated(&self, data: GetUsersPaginated) -> Result<Response<String>, Error> {
        let users = self
            .repository
            .get_paginated(
                data.page.unwrap_or(1_u16),
                data.per_page.unwrap_or(25),
                data.sort_by,
            )
            .await?;

        Ok(UserResponse::new(users)
            .into_response(StatusCode::OK)
            .json()?)
    }
}
