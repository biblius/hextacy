use super::{
    contract::ServiceContract,
    data::{GetUsersPaginated, UserResponse},
};
use crate::error::Error;
use actix_web::HttpResponse;
use async_trait::async_trait;
use infrastructure::web::http::response::Response;
use reqwest::StatusCode;
use storage::repository::user::UserRepository;

pub(super) struct UserService<R: UserRepository> {
    pub repository: R,
}

#[async_trait]
impl<R> ServiceContract for UserService<R>
where
    R: UserRepository + Send + Sync,
{
    fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error> {
        let users = self.repository.get_paginated(
            data.page.unwrap_or(1_u16),
            data.per_page.unwrap_or(25),
            data.sort_by,
        )?;

        Ok(UserResponse::new(users)
            .to_response(StatusCode::OK)
            .finish())
    }
}
