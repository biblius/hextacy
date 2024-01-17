use crate::{
    core::models::{session::Session, user::User},
    db::adapters::AdapterError,
};
use std::future::Future;
use uuid::Uuid;

pub trait UserRepository {
    fn get_by_id(
        &self,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<User>, AdapterError>> + Send;

    fn get_by_username(
        &self,
        username: &str,
    ) -> impl Future<Output = Result<Option<User>, AdapterError>> + Send;

    fn create(
        &self,
        username: &str,
        password: &str,
    ) -> impl Future<Output = Result<User, AdapterError>> + Send;

    async fn insert_with_session(
        &self,
        username: &str,
        password: &str,
        expires: bool,
    ) -> Result<(User, Session), AdapterError>;
}
