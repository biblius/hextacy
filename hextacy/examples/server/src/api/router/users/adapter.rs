use super::contract::RepositoryContract;
use crate::db::models::user;
use crate::db::repository::user::UserRepository;
use crate::error::Error;
use hextacy::clients::db::{postgres::PgPoolConnection, DBConnect};
use hextacy::db::RepositoryAccess;
use hextacy::{contract, repository};

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
