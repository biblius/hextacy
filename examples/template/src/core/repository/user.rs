use crate::{core::models::user::User, db::adapters::AdapterError};
use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository<C> {
    async fn get_by_id(&self, conn: &mut C, id: Uuid) -> Result<Option<User>, AdapterError>;

    async fn get_by_username(
        &self,
        conn: &mut C,
        username: &str,
    ) -> Result<Option<User>, AdapterError>;

    async fn create(
        &self,
        conn: &mut C,
        username: &str,
        password: &str,
    ) -> Result<User, AdapterError>;
}
