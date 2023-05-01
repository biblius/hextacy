use hextacy::{
    derive::Adapter,
    drivers::db::{postgres::diesel::PgPoolConnection, DBConnect, Driver},
};
trait UserRepository<C> {}

#[derive(Adapter)]
struct ServiceRepo<D, C, User>
where
    D: DBConnect<Connection = C> + Send + Sync,
    User: Send + Sync,
{
    #[diesel(C)]
    driver: Driver<D, C>,
    _user: User,
}

fn main() {}
