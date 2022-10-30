use super::contract::{CacheContract, RepositoryContract};
use crate::error::Error;
use crate::helpers::cache::{Cache as CacheService, CacheId};
use async_trait::async_trait;
use chrono::Utc;
use infrastructure::{
    clients::store::redis::{Commands, Redis},
    config::constants::SESSION_CACHE_DURATION_SECONDS,
    store::{
        adapters::{postgres::PgAdapterError, AdapterError},
        models::user_session::UserSession,
        repository::{
            session::{Session, SessionRepository},
            user::UserRepository,
        },
    },
};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Repository<SR: SessionRepository, UR: UserRepository> {
    pub session_repo: SR,
    pub user_repo: UR,
}

#[async_trait]
impl<SR, UR> RepositoryContract for Repository<SR, UR>
where
    SR: SessionRepository<Error = PgAdapterError> + Send + Sync,
    UR: UserRepository<Error = PgAdapterError> + Send + Sync,
{
    /// Attempts to find an unexpired session with its corresponding CSRF
    async fn get_valid_user_session(&self, id: &str, csrf: &str) -> Result<UserSession, Error> {
        let session = self
            .session_repo
            .get_valid_by_id(id, csrf)
            .await
            .map_err(AdapterError::Postgres)?;
        let user = self
            .user_repo
            .get_by_id(&session.user_id)
            .await
            .map_err(AdapterError::Postgres)?;
        Ok(UserSession::new(user, session))
    }

    /// Extends session `expires_at` for 30 minutes
    async fn refresh_session(&self, id: &str, csrf: &str) -> Result<Session, Error> {
        self.session_repo
            .refresh(id, csrf)
            .await
            .map_err(|e| AdapterError::Postgres(e).into())
    }
}

#[derive(Debug, Clone)]
pub struct Cache {
    pub client: Arc<Redis>,
}

#[async_trait]
impl CacheContract for Cache {
    async fn get_session_by_id(&self, id: &str) -> Result<UserSession, Error> {
        CacheService::get(CacheId::Session, id, &mut self.client.connect()?).map_err(Error::new)
    }

    async fn cache_session(&self, id: &str, session: &UserSession) -> Result<(), Error> {
        CacheService::set(
            CacheId::Session,
            id,
            session,
            Some(SESSION_CACHE_DURATION_SECONDS),
            &mut self.client.connect()?,
        )
        .map_err(Error::new)
    }

    async fn refresh_session(&self, session_id: &str) -> Result<(), Error> {
        let mut conn = self.client.connect()?;
        conn.expire_at(
            session_id,
            ((Utc::now().timestamp() + SESSION_CACHE_DURATION_SECONDS as i64) % i64::MAX) as usize,
        )?;
        Ok(())
    }
}
