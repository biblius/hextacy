use super::contract::RepositoryContract;
use crate::error::Error;
use async_trait::async_trait;
use infrastructure::store::{
    adapters::{postgres::PgAdapterError, AdapterError},
    repository::user::{SortOptions, User, UserRepository},
};

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
            .map_err(|e| AdapterError::Postgres(e).into())
    }
}
