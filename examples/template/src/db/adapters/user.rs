use super::super::entities::{users::ActiveModel as UserModel, users::Entity as UserEntity};
use crate::core::models::session::Session;
use crate::core::models::user::User;
use crate::core::repository::user::UserRepository;
use crate::db::adapters::AdapterError;
use crate::db::driver::SeaormDriver;
use crate::db::entities::sessions::ActiveModel as SessionModel;
use crate::db::entities::sessions::Entity as SessionEntity;
use crate::db::entities::users::Column;
use async_trait::async_trait;
use hextacy::transaction;
use hextacy::Atomic;
use hextacy::Driver;
use sea_orm::prelude::*;
use sea_orm::ConnectionTrait;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserAdapter {
    pub driver: SeaormDriver,
}

impl UserRepository for UserAdapter {
    async fn get_by_id(&self, id: Uuid) -> Result<Option<User>, AdapterError> {
        let conn = self.driver.connect().await?;

        UserEntity::find_by_id(id)
            .one(&conn)
            .await
            .map_err(AdapterError::SeaORM)
            .map(|u| u.map(User::from))
    }

    async fn get_by_username(&self, username: &str) -> Result<Option<User>, AdapterError> {
        let conn = self.driver.connect().await?;
        UserEntity::find()
            .filter(Column::Username.eq(username))
            .one(&conn)
            .await
            .map_err(AdapterError::SeaORM)
            .map(|user| user.map(User::from))
    }

    async fn create(&self, username: &str, password: &str) -> Result<User, AdapterError> {
        let conn = self.driver.connect().await?;
        let user: UserModel = User::new(username.to_string(), password.to_string()).into();
        UserEntity::insert(user)
            .exec_with_returning(&conn)
            .await
            .map(User::from)
            .map_err(AdapterError::SeaORM)
    }

    async fn insert_with_session(
        &self,
        username: &str,
        password: &str,
        expires: bool,
    ) -> Result<(User, Session), AdapterError> {
        let conn = self.driver.connect().await?;

        let user = User::new(username.to_string(), password.to_string());

        let session: SessionModel = Session::new(user.id, expires).into();
        let user: UserModel = user.into();

        let (user, session) = transaction!(
            conn: DatabaseConnection => {
                let user = UserEntity::insert(user)
                    .exec_with_returning(&conn)
                    .await
                    .map(User::from)
                    .map_err(AdapterError::SeaORM)?;

                let session = SessionEntity::insert(session)
                    .exec_with_returning(&conn)
                    .await
                    .map(Session::from)
                    .map_err(AdapterError::SeaORM)?;

                Ok((user, session))
            }
        )?;

        Ok((user, session))
    }
}
