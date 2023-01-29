use clients::postgres::PgPoolConnection;
use diesel::PgConnection;

pub mod session;
pub mod user;

pub type TransactionCallback<T, E> = Box<dyn FnOnce(&mut PgConnection) -> Result<T, E>>;

pub trait PgRepositoryAccess {
    fn transaction<T, E: From<diesel::result::Error>>(
        &self,
        cb: TransactionCallback<T, E>,
    ) -> Result<T, E> {
        self.connect()
            .build_transaction()
            .deferrable()
            .run(|conn| cb(conn))
    }

    fn connect(&self) -> PgPoolConnection;
}
