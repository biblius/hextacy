pub mod adapters;
pub mod models;
pub mod repository;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoAdapterError {
    #[error("Entry does not exist")]
    DoesNotExist,
    #[error("Diesel: {0}")]
    Diesel(diesel::result::Error),
    #[error("Mongo: {0}")]
    Mongo(#[from] mongodb::error::Error),
    #[error("SeaORM: {0}")]
    SeaORM(#[from] sea_orm::DbErr),
}

impl From<diesel::result::Error> for RepoAdapterError {
    fn from(value: diesel::result::Error) -> Self {
        match value {
            diesel::result::Error::NotFound => Self::DoesNotExist,
            e => Self::Diesel(e),
        }
    }
}
