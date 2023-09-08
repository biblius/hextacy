use crate::db::models::user;
use crate::db::repository::user::UserRepository;
use crate::error::Error;
use hextacy::{component, contract, Driver};

pub struct UsersRepository<A, C, User>
where
    A: Driver<Connection = C>,
    User: UserRepository<C>,
{
    driver: A,
    _user: std::marker::PhantomData<User>,
}

impl<A, C, User> UsersRepository<A, C, User>
where
    A: Driver<Connection = C>,
    User: UserRepository<C>,
{
    pub fn new(driver: A) -> Self {
        Self {
            driver,
            _user: std::marker::PhantomData,
        }
    }
}

#[component(
    use Driver for Connection:Atomic,
    use UserRepository with Connection as User
)]
#[contract]
impl UsersRepository {
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

/* where
Conn: Send,
User: UserRepository<Conn> + Send + Sync,
D: Driver<Connection = Conn> + Send + Sync, */
