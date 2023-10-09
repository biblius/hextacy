use super::super::entities::{users::ActiveModel as UserModel, users::Entity as UserEntity};
use crate::core::models::user::User;
use crate::core::repository::user::UserRepository;
use crate::db::adapters::AdapterError;
use crate::db::entities::users::Column;
use async_trait::async_trait;
use hextacy::exports::uuid::Uuid;
use sea_orm::prelude::*;
use sea_orm::ConnectionTrait;

#[derive(Debug, Clone)]
pub struct UserAdapter;

#[async_trait]
impl<C> UserRepository<C> for UserAdapter
where
    C: ConnectionTrait + Send + Sync,
{
    async fn get_by_id(&self, conn: &mut C, id: Uuid) -> Result<Option<User>, AdapterError> {
        UserEntity::find_by_id(id)
            .one(conn)
            .await
            .map_err(AdapterError::SeaORM)
            .map(|u| u.map(User::from))
    }

    async fn get_by_username(
        &self,
        conn: &mut C,
        username: &str,
    ) -> Result<Option<User>, AdapterError> {
        UserEntity::find()
            .filter(Column::Username.eq(username))
            .one(conn)
            .await
            .map_err(AdapterError::SeaORM)
            .map(|user| user.map(User::from))
    }

    async fn create(
        &self,
        conn: &mut C,
        username: &str,
        password: &str,
    ) -> Result<User, AdapterError> {
        let user: UserModel = User::new(username.to_string(), password.to_string()).into();
        UserEntity::insert(user)
            .exec_with_returning(conn)
            .await
            .map(User::from)
            .map_err(AdapterError::SeaORM)
    }
}
