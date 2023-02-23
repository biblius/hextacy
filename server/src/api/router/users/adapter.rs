use super::contract::RepositoryContract;
use crate::error::Error;
use alx_core::clients::db::postgres::{PgPoolConnection, Postgres};
use std::{marker::PhantomData, sync::Arc};
use storage::{repository::user::UserRepository, RepoAccess};

pub(super) struct Repository<User, C>
where
    User: UserRepository<C>,
{
    pub client: Arc<Postgres>,
    user: PhantomData<User>,
    conn: PhantomData<C>,
}

impl<User, C> Repository<User, C>
where
    User: UserRepository<C>,
{
    pub fn new(client: Arc<Postgres>) -> Self {
        Self {
            client,
            user: PhantomData,
            conn: PhantomData,
        }
    }
}

impl<User> RepoAccess<PgPoolConnection> for Repository<User, PgPoolConnection>
where
    User: UserRepository<PgPoolConnection>,
{
    fn connect(&self) -> Result<PgPoolConnection, storage::adapters::AdapterError> {
        self.client.connect().map_err(|e| e.into())
    }
}

impl<User, Conn> RepositoryContract for Repository<User, Conn>
where
    Self: RepoAccess<Conn>,
    User: UserRepository<Conn>,
{
    fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<storage::models::user::SortOptions>,
    ) -> Result<Vec<storage::models::user::User>, Error> {
        let mut conn = self.connect()?;
        User::get_paginated(&mut conn, page, per_page, sort).map_err(Error::new)
    }
}
