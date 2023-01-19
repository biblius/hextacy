use super::RepositoryError;
use crate::storage::models::{session::Session, user::User};

#[mockall::automock]
pub trait SessionRepository {
    /// Create a session
    fn create(&self, user: &User, csrf: &str, permanent: bool) -> Result<Session, RepositoryError>;

    /// Get unexpired session corresponding to the CSRF token
    fn get_valid_by_id(&self, id: &str, csrf: &str) -> Result<Session, RepositoryError>;

    /// Update session's `expires_at` field
    fn refresh(&self, id: &str, csrf: &str) -> Result<Session, RepositoryError>;

    /// Update session's `expires_at` field to now
    fn expire(&self, id: &str) -> Result<Session, RepositoryError>;

    /// Expire all user sessions. A session ID can be provided to skip purging a specific session.
    fn purge<'a>(
        &self,
        user_id: &str,
        skip: Option<&'a str>,
    ) -> Result<Vec<Session>, RepositoryError>;
}
