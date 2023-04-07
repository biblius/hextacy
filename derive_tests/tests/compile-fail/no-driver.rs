use hextacy::{
    derive::Repository,
    drivers::{
        db::{DBConnect, Driver},
        DriverError,
    },
};

trait UserRepository<C> {}
struct Conn;
struct SomeDriver;

#[async_trait::async_trait]
impl DBConnect for SomeDriver {
    type Connection = Conn;
    async fn connect(&self) -> Result<Self::Connection, DriverError> {
        Ok(Conn)
    }
}

#[derive(Repository)]
struct ServiceRepo<D, C, User>
where
    D: DBConnect<Connection = C>,
{
    driver: Driver<D, C>,
    _user: User,
}

fn main() {}
