pub mod session;
pub mod user;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Driver: {0}")]
    Driver(#[from] hextacy::DriverError),
    #[error("SeaORM: {0}")]
    SeaORM(#[from] sea_orm::DbErr),
}
