use super::data::GetUsersPaginated;
use crate::db::models::user::{self, User};
use crate::error::Error;
use async_trait::async_trait;
use hextacy::web::http::Response;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait ServiceContract {
    async fn get_paginated(&self, data: GetUsersPaginated) -> Result<Response<String>, Error>;
}
#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait RepositoryContract {
    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<user::SortOptions>,
    ) -> Result<Vec<User>, Error>;
}
