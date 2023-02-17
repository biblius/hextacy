use alx_clients::oauth::OAuthProvider;

use crate::{
    adapters::AdapterError,
    models::{session::Session, user::User},
};

#[mockall::automock]
pub trait SessionRepository {
    /// Create a session
    fn create<'a>(
        &self,
        user: &User,
        csrf: &str,
        expires_after: Option<i64>,
        oauth_token: Option<&'a str>,
        provider: Option<OAuthProvider>,
    ) -> Result<Session, AdapterError>;

    /// Get unexpired session corresponding to the CSRF token
    fn get_valid_by_id(&self, id: &str, csrf: &str) -> Result<Session, AdapterError>;

    /// Update session's `expires_at` field
    fn refresh(&self, id: &str, csrf: &str) -> Result<Session, AdapterError>;

    /// Update session's `expires_at` field to now
    fn expire(&self, id: &str) -> Result<Session, AdapterError>;

    /// Expire all user sessions. A session ID can be provided to skip purging a specific session.
    fn purge<'a>(&self, user_id: &str, skip: Option<&'a str>)
        -> Result<Vec<Session>, AdapterError>;

    /// Update all sessions' OAuth access tokens based on the user ID and the provider.
    fn update_access_tokens(
        &self,
        access_token: &str,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<Vec<Session>, AdapterError>;
}
