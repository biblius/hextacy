pub mod session;
pub mod user;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("SeaORM: {0}")]
    SeaORM(#[from] sea_orm::DbErr),
}
