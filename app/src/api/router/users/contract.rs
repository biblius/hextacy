use super::data::GetUsersPaginated;
use crate::error::Error;
use actix_web::HttpResponse;
use async_trait::async_trait;
use infrastructure::repository::user::{SortOptions, User};

#[async_trait]
pub(super) trait ServiceContract {
    async fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error>;
}

#[async_trait]
pub(super) trait RepositoryContract {
    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort_by: Option<SortOptions>,
    ) -> Result<Vec<User>, Error>;
}
