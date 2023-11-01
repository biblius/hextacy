#![allow(unused_variables)]

use crate::{
    db::{
        models::{session::Session, user::User},
        repository::session::SessionRepository,
        RepoAdapterError,
    },
    services::oauth::OAuthProvider,
};
use async_trait::async_trait;
use mongodb::{bson::doc, ClientSession};

#[derive(Debug, Clone)]
pub struct MgSessionAdapter;

#[async_trait]
impl SessionRepository<ClientSession> for MgSessionAdapter {
    async fn create(
        conn: &mut ClientSession,
        user: &User,
        csrf: &str,
        expires_after: Option<i64>,
        oauth_token: Option<&str>,
        provider: Option<OAuthProvider>,
    ) -> Result<Session, RepoAdapterError> {
        todo!()
    }

    /// Get unexpired session corresponding to the CSRF token
    async fn get_valid_by_id(
        conn: &mut ClientSession,
        id: &str,
        csrf: &str,
    ) -> Result<Session, RepoAdapterError> {
        todo!()
    }

    /// Update session's `expires_at` field
    async fn refresh(
        conn: &mut ClientSession,
        id: &str,
        csrf: &str,
    ) -> Result<Session, RepoAdapterError> {
        todo!()
    }

    /// Update session's `expires_at` field to now
    async fn expire(conn: &mut ClientSession, id: &str) -> Result<Session, RepoAdapterError> {
        todo!()
    }

    /// Expire all user sessions. A session ID can be provided to skip purging a specific session.
    async fn purge(
        conn: &mut ClientSession,
        user_id: &str,
        skip: Option<&str>,
    ) -> Result<Vec<Session>, RepoAdapterError> {
        todo!()
    }

    /// Update all sessions' OAuth access tokens based on the user ID and the provider.
    async fn update_access_tokens(
        conn: &mut ClientSession,
        access_token: &str,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<Vec<Session>, RepoAdapterError> {
        todo!()
    }
}
