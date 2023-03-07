use super::contract::RepositoryContract;
use crate::error::Error;
use alx_core::clients::db::{postgres::PgPoolConnection, DBConnect};
use alx_core::db::RepositoryAccess;
use alx_core::{contract, repository};
use storage::models::user;
use storage::repository::user::UserRepository;

repository! {
    Postgres => PgConnection : postgres;

    User => UserRepository<PgConnection>
}

contract! {
    Postgres => PgConnection;

    RepositoryContract => Repository, RepositoryAccess;

    User => UserRepository<PgConnection>;

    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<user::SortOptions>,
    ) -> Result<Vec<user::User>, Error> {
        let mut conn = self.connect().await?;
        User::get_paginated(&mut conn, page, per_page, sort).await.map_err(Error::new)
    }
}
