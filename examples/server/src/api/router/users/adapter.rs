use super::contract::RepositoryContract;
use crate::db::models::user;
use crate::db::repository::user::UserRepository;
use crate::error::Error;
use hextacy::drivers::Connect;
use hextacy::drivers::Driver;
use std::sync::Arc;

pub struct Repository<A, C, User>
where
    A: Connect<Connection = C>,
    User: UserRepository<C>,
{
    driver: Driver<A, C>,
    _user: std::marker::PhantomData<User>,
}

impl<A, C, User> Repository<A, C, User>
where
    A: Connect<Connection = C>,
    User: UserRepository<C>,
{
    pub fn new(driver: Arc<A>) -> Self {
        Self {
            driver: Driver::new(driver),
            _user: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<Driver, Conn, User> RepositoryContract for Repository<Driver, Conn, User>
where
    Conn: Send,
    User: UserRepository<Conn> + Send + Sync,
    Driver: Connect<Connection = Conn> + Send + Sync,
{
    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<user::SortOptions>,
    ) -> Result<Vec<user::User>, Error> {
        let mut conn = self.driver.connect().await?;

        User::get_paginated(&mut conn, page, per_page, sort)
            .await
            .map_err(Error::new)
    }
}
