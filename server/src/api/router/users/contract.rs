use super::data::GetUsersPaginated;
use crate::error::Error;
use actix_web::HttpResponse;
use storage::models::user::{self, User};

pub(super) trait ServiceContract {
    fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error>;
}

pub(super) trait RepositoryContract {
    fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<user::SortOptions>,
    ) -> Result<Vec<User>, Error>;
}
