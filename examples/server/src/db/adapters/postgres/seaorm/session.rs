#![allow(unused_variables)]
use crate::db::{
    adapters::AdapterError,
    models::{session::Session, user::User},
    repository::session::SessionRepository,
};
use crate::services::oauth::OAuthProvider;
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use sea_orm::{prelude::*, DatabaseTransaction};

#[derive(Debug, Clone)]
pub struct PgSessionAdapter;

#[async_trait]
impl SessionRepository<DatabaseConnection> for PgSessionAdapter {
    /// Create a session
    async fn create(
        conn: &mut DatabaseConnection,
        user: &User,
        csrf: &str,
        expires_after: Option<i64>,
        oauth_token: Option<&str>,
        provider: Option<OAuthProvider>,
    ) -> Result<Session, AdapterError> {
        todo!()
    }

    /// Get unexpired session corresponding to the CSRF token
    async fn get_valid_by_id(
        conn: &mut DatabaseConnection,
        id: &str,
        csrf: &str,
    ) -> Result<Session, AdapterError> {
        todo!()
    }

    /// Update session's `expires_at` field
    async fn refresh(
        conn: &mut DatabaseConnection,
        id: &str,
        csrf: &str,
    ) -> Result<Session, AdapterError> {
        todo!()
    }

    /// Update session's `expires_at` field to now
    async fn expire(conn: &mut DatabaseConnection, id: &str) -> Result<Session, AdapterError> {
        todo!()
    }

    /// Expire all user sessions. A session ID can be provided to skip purging a specific session.
    async fn purge(
        conn: &mut DatabaseConnection,
        user_id: &str,
        skip: Option<&str>,
    ) -> Result<Vec<Session>, AdapterError> {
        todo!()
    }

    /// Update all sessions' OAuth access tokens based on the user ID and the provider.
    async fn update_access_tokens(
        conn: &mut DatabaseConnection,
        access_token: &str,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<Vec<Session>, AdapterError> {
        todo!()
    }
}

#[async_trait]
impl SessionRepository<DatabaseTransaction> for PgSessionAdapter {
    /// Create a session
    async fn create(
        conn: &mut DatabaseTransaction,
        user: &User,
        csrf: &str,
        expires_after: Option<i64>,
        oauth_token: Option<&str>,
        provider: Option<OAuthProvider>,
    ) -> Result<Session, AdapterError> {
        todo!()
    }

    /// Get unexpired session corresponding to the CSRF token
    async fn get_valid_by_id(
        conn: &mut DatabaseTransaction,
        id: &str,
        csrf: &str,
    ) -> Result<Session, AdapterError> {
        todo!()
    }

    /// Update session's `expires_at` field
    async fn refresh(
        conn: &mut DatabaseTransaction,
        id: &str,
        csrf: &str,
    ) -> Result<Session, AdapterError> {
        todo!()
    }

    /// Update session's `expires_at` field to now
    async fn expire(conn: &mut DatabaseTransaction, id: &str) -> Result<Session, AdapterError> {
        todo!()
    }

    /// Expire all user sessions. A session ID can be provided to skip purging a specific session.
    async fn purge(
        conn: &mut DatabaseTransaction,
        user_id: &str,
        skip: Option<&str>,
    ) -> Result<Vec<Session>, AdapterError> {
        todo!()
    }

    /// Update all sessions' OAuth access tokens based on the user ID and the provider.
    async fn update_access_tokens(
        conn: &mut DatabaseTransaction,
        access_token: &str,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<Vec<Session>, AdapterError> {
        todo!()
    }
}
