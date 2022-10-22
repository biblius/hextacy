use crate::error::Error;
use async_trait::async_trait;
use infrastructure::{
    adapters::postgres::PgAdapterError,
    repository::user::{SortOptions, User, UserRepository},
};

use super::contract::RepositoryContract;

pub(super) struct Repository<UR>
where
    UR: UserRepository,
{
    pub user_repo: UR,
}

#[async_trait]
impl<UR> RepositoryContract for Repository<UR>
where
    UR: UserRepository<Error = PgAdapterError> + Send + Sync,
{
    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort_by: Option<SortOptions>,
    ) -> Result<Vec<User>, Error> {
        self.user_repo
            .get_paginated(page, per_page, sort_by)
            .await
            .map_err(Error::new)
    }
}
