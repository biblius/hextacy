use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;

//pub fn build_pool() -> PgPool {}

pub struct Pg {}
