use crate::{
    core::models::{session::Session, user::User},
    db::adapters::AdapterError,
};
use async_trait::async_trait;
use hextacy::Driver;
use uuid::Uuid;

pub trait SessionRepository {
    async fn get_valid_by_id(&self, id: Uuid, csrf: Uuid) -> Result<Option<Session>, AdapterError>;
    async fn create(&self, user: &User, expires: bool) -> Result<Session, AdapterError>;
    async fn expire(&self, id: Uuid) -> Result<Session, AdapterError>;
    async fn purge(&self, user_id: Uuid) -> Result<u64, AdapterError>;
}
