use hextacy::{
    derive::Adapter,
    drivers::db::{postgres::diesel::DieselConnection, Connect, Driver},
};
trait UserRepository<C> {}

#[derive(Adapter)]
struct ServiceRepo<D, C, User>
where
    D: Connect<Connection = C> + Send + Sync,
    User: Send + Sync,
{
    #[diesel(C)]
    driver: Driver<D, C>,
    _user: User,
}

fn main() {}
