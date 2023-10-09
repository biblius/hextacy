use super::super::entities::sessions::{
    ActiveModel as SessionModel, Column, Entity as SessionEntity,
};
use crate::core::models::session::Session;
use crate::core::models::user::User;
use crate::core::repository::session::SessionRepository;
use crate::db::adapters::AdapterError;
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::prelude::*;
use sea_orm::ConnectionTrait;
use sea_orm::Set;

#[derive(Debug, Clone)]
pub struct SessionAdapter;

#[async_trait]
impl<C> SessionRepository<C> for SessionAdapter
where
    C: ConnectionTrait + Send + Sync,
{
    async fn get_valid_by_id(
        &self,
        conn: &mut C,
        id: Uuid,
        csrf: Uuid,
    ) -> Result<Option<Session>, AdapterError> {
        SessionEntity::find()
            .filter(Column::Id.eq(id))
            .filter(Column::Csrf.eq(csrf))
            .filter(Column::ExpiresAt.gt(Utc::now()))
            .one(conn)
            .await
            .map(|s| s.map(Session::from))
            .map_err(AdapterError::SeaORM)
    }

    async fn create(
        &self,
        conn: &mut C,
        user: &User,
        expires: bool,
    ) -> Result<Session, AdapterError> {
        let session: SessionModel = Session::new(user.id, expires).into();
        SessionEntity::insert(session)
            .exec_with_returning(conn)
            .await
            .map(Session::from)
            .map_err(AdapterError::SeaORM)
    }

    async fn expire(&self, conn: &mut C, id: Uuid) -> Result<Session, AdapterError> {
        SessionModel {
            id: Set(id),
            expires_at: Set(Utc::now().into()),
            ..Default::default()
        }
        .update(conn)
        .await
        .map(Session::from)
        .map_err(AdapterError::SeaORM)
    }

    async fn purge(&self, conn: &mut C, user_id: Uuid) -> Result<u64, AdapterError> {
        SessionEntity::update_many()
            .col_expr(Column::ExpiresAt, Expr::value(Utc::now()))
            .filter(Column::UserId.eq(user_id))
            .exec(conn)
            .await
            .map(|res| res.rows_affected)
            .map_err(AdapterError::SeaORM)
    }
}
