use super::data::GetUsersPaginated;
use crate::db::models::user::{self, User};
use crate::error::Error;
use actix_web::HttpResponse;
use async_trait::async_trait;

#[async_trait(?Send)]
pub(super) trait ServiceContract {
    async fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error>;
}
#[async_trait(?Send)]
pub(super) trait RepositoryContract {
    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<user::SortOptions>,
    ) -> Result<Vec<User>, Error>;
}
