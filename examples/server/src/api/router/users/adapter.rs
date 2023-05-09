use super::contract::RepositoryContract;
use crate::db::models::user;
use crate::db::repository::user::UserRepository;
use crate::error::Error;
use async_trait::async_trait;
use hextacy::db::{DatabaseError, RepositoryAccess};
use hextacy::drivers::db::Connect;
use hextacy::drivers::db::Driver;
use std::sync::Arc;

pub struct Repository<A, C, User>
where
    A: Connect<Connection = C>,
    User: UserRepository<C>,
{
    postgres: Driver<A, C>,
    _user: std::marker::PhantomData<User>,
}

impl<A, C, User> Repository<A, C, User>
where
    A: Connect<Connection = C>,
    User: UserRepository<C>,
{
    pub fn new(driver: Arc<A>) -> Self {
        Self {
            postgres: Driver::new(driver),
            _user: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<A, C, User> RepositoryAccess<C> for Repository<A, C, User>
where
    A: Connect<Connection = C> + Send + Sync,
    User: UserRepository<C> + Send + Sync,
{
    async fn connect(&self) -> Result<C, DatabaseError> {
        self.postgres.connect().await.map_err(DatabaseError::from)
    }
}

#[async_trait::async_trait]
impl<Driver, Conn, User> RepositoryContract for Repository<Driver, Conn, User>
where
    Self: RepositoryAccess<Conn>,
    Driver: Connect<Connection = Conn> + Send + Sync,
    User: UserRepository<Conn> + Send + Sync,
    Conn: Send,
{
    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<user::SortOptions>,
    ) -> Result<Vec<user::User>, Error> {
        let mut conn = self.connect().await?;

        User::get_paginated(&mut conn, page, per_page, sort)
            .await
            .map_err(Error::new)
    }
}
