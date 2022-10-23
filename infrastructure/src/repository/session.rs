use super::user::User;
use super::{role::Role, RepositoryError};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub user_role: Role,
    #[serde(skip)]
    pub csrf_token: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

impl Session {
    #[inline]
    /// Check whether the session has an expiration time
    pub fn is_permanent(&self) -> bool {
        self.expires_at.timestamp() == NaiveDateTime::MAX.timestamp()
    }

    pub fn __mock(id: String, user: &User, csrf: String, permanent: bool) -> Self {
        Self {
            id,
            user_id: user.id.clone(),
            username: user.username.clone(),
            user_role: user.role.clone(),
            csrf_token: csrf,
            created_at: NaiveDateTime::from_timestamp(chrono::Utc::now().timestamp(), 0),
            updated_at: NaiveDateTime::from_timestamp(chrono::Utc::now().timestamp(), 0),
            expires_at: if permanent {
                NaiveDateTime::MAX
            } else {
                NaiveDateTime::from_timestamp(chrono::Utc::now().timestamp(), 0)
                    + chrono::Duration::minutes(30)
            },
        }
    }
}

#[async_trait]
pub trait SessionRepository {
    type Error: Error + Into<RepositoryError>;

    /// Create a session
    async fn create(
        &self,
        user: &User,
        csrf: &str,
        permanent: bool,
    ) -> Result<Session, Self::Error>;

    /// Get unexpired session corresponding to the CSRF token
    async fn get_valid_by_id(&self, id: &str, csrf: &str) -> Result<Session, Self::Error>;

    /// Update session's `expires_at` field
    async fn refresh(&self, id: &str, csrf: &str) -> Result<Session, Self::Error>;

    /// Update session's `expires_at` field to now
    async fn expire(&self, id: &str) -> Result<Session, Self::Error>;

    /// Expire all user sessions. A session ID can be provided to skip purging a specific session.
    async fn purge<'a>(
        &self,
        user_id: &str,
        skip: Option<&'a str>,
    ) -> Result<Vec<Session>, Self::Error>;
}
