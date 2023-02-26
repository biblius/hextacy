use super::contract::RepositoryContract;
use crate::error::Error;
use alx_core::clients::db::{postgres::PgPoolConnection, DBConnect};
use alx_core::db::RepoAccess;
use alx_core::{contract, pg_repo};
use storage::models::user;
use storage::repository::user::UserRepository;

pg_repo! {
    Repository,

    Conn => "Conn",

    User => UserRepository<Conn>
}

contract! {
    RepositoryContract => Repository,
    RepoAccess,
    User => UserRepository<C>;

    fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<user::SortOptions>,
    ) -> Result<Vec<user::User>, Error> {
        let mut conn = self.connect()?;
        User::get_paginated(&mut conn, page, per_page, sort).map_err(Error::new)
    }
}
