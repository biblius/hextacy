use crate::{
    core::models::{session::Session, user::User},
    db::adapters::AdapterError,
};
use async_trait::async_trait;
use hextacy::exports::uuid::Uuid;

#[async_trait]
pub trait SessionRepository<C> {
    async fn get_valid_by_id(
        &self,
        conn: &mut C,
        id: Uuid,
        csrf: Uuid,
    ) -> Result<Option<Session>, AdapterError>;
    async fn create(
        &self,
        conn: &mut C,
        user: &User,
        expires: bool,
    ) -> Result<Session, AdapterError>;
    async fn expire(&self, conn: &mut C, id: Uuid) -> Result<Session, AdapterError>;
    async fn purge(&self, conn: &mut C, user_id: Uuid) -> Result<u64, AdapterError>;
}
