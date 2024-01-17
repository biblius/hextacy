use super::super::entities::sessions::{
    ActiveModel as SessionModel, Column, Entity as SessionEntity,
};
use crate::core::models::session::Session;
use crate::core::models::user::User;
use crate::core::repository::session::SessionRepository;
use crate::db::adapters::AdapterError;
use crate::db::driver::SeaormDriver;
use chrono::Utc;
use hextacy::Atomic;
use hextacy::Driver;
use sea_orm::prelude::*;
use sea_orm::Set;

#[derive(Debug, Clone)]
pub struct SessionAdapter {
    pub driver: SeaormDriver,
}

impl SessionRepository for SessionAdapter {
    async fn get_valid_by_id(&self, id: Uuid, csrf: Uuid) -> Result<Option<Session>, AdapterError> {
        let conn = self.driver.connect().await?;
        SessionEntity::find()
            .filter(Column::Id.eq(id))
            .filter(Column::Csrf.eq(csrf))
            .filter(Column::ExpiresAt.gt(Utc::now()))
            .one(&conn)
            .await
            .map(|s| s.map(Session::from))
            .map_err(AdapterError::SeaORM)
    }

    async fn create(&self, user: &User, expires: bool) -> Result<Session, AdapterError> {
        let conn = self.driver.connect().await?;
        let session: SessionModel = Session::new(user.id, expires).into();
        SessionEntity::insert(session)
            .exec_with_returning(&conn)
            .await
            .map(Session::from)
            .map_err(AdapterError::SeaORM)
    }

    async fn expire(&self, id: Uuid) -> Result<Session, AdapterError> {
        let conn = self.driver.connect().await?;
        SessionModel {
            id: Set(id),
            expires_at: Set(Utc::now().into()),
            ..Default::default()
        }
        .update(&conn)
        .await
        .map(Session::from)
        .map_err(AdapterError::SeaORM)
    }

    async fn purge(&self, user_id: Uuid) -> Result<u64, AdapterError> {
        let conn = self.driver.connect().await?;
        SessionEntity::update_many()
            .col_expr(Column::ExpiresAt, Expr::value(Utc::now()))
            .filter(Column::UserId.eq(user_id))
            .exec(&conn)
            .await
            .map(|res| res.rows_affected)
            .map_err(AdapterError::SeaORM)
    }
}
