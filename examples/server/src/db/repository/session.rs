use async_trait::async_trait;

use crate::{
    db::{
        models::{session::Session, user::User},
        RepoAdapterError,
    },
    services::oauth::OAuthProvider,
};

#[async_trait]
pub trait SessionRepository<C> {
    /// Create a session
    async fn create(
        conn: &mut C,
        user: &User,
        csrf: &str,
        expires_after: Option<i64>,
        oauth_token: Option<&str>,
        provider: Option<OAuthProvider>,
    ) -> Result<Session, RepoAdapterError>;

    /// Get unexpired session corresponding to the CSRF token
    async fn get_valid_by_id(
        conn: &mut C,
        id: &str,
        csrf: &str,
    ) -> Result<Session, RepoAdapterError>;

    /// Update session's `expires_at` field
    async fn refresh(conn: &mut C, id: &str, csrf: &str) -> Result<Session, RepoAdapterError>;

    /// Update session's `expires_at` field to now
    async fn expire(conn: &mut C, id: &str) -> Result<Session, RepoAdapterError>;

    /// Expire all user sessions. A session ID can be provided to skip purging a specific session.
    async fn purge(
        conn: &mut C,
        user_id: &str,
        skip: Option<&str>,
    ) -> Result<Vec<Session>, RepoAdapterError>;

    /// Update all sessions' OAuth access tokens based on the user ID and the provider.
    async fn update_access_tokens(
        conn: &mut C,
        access_token: &str,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<Vec<Session>, RepoAdapterError>;
}
