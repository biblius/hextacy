pub mod mongo;
pub mod postgres;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Entry does not exist")]
    DoesNotExist,
    #[error("Driver: {0}")]
    Driver(#[from] hextacy::drivers::DriverError),
    #[error("Diesel: {0}")]
    Diesel(diesel::result::Error),
    #[error("Mongo: {0}")]
    Mongo(#[from] mongodb::error::Error),
    #[error("SeaORM: {0}")]
    SeaORM(#[from] sea_orm::DbErr),
}

impl From<diesel::result::Error> for AdapterError {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => Self::DoesNotExist,
            e => Self::Diesel(e),
        }
    }
}
