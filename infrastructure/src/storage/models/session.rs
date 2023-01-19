use super::user::User;
use crate::storage::repository::role::Role;
use chrono::NaiveDateTime;
use diesel::Queryable;
use serde::{Deserialize, Serialize};

/// The repository session model
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

/// A cacheable struct with a session and its user appended to it for quick access.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserSession {
    pub id: String,
    pub csrf: String,
    pub user_id: String,
    pub user_role: Role,
    pub user_email: String,
    pub user_name: String,
    pub user_phone: Option<String>,
    pub frozen: bool,
    pub google_id: Option<String>,
    pub github_id: Option<String>,
    pub expires_at: i64,
}

impl UserSession {
    pub fn new(user: User, session: Session) -> Self {
        Self {
            id: session.id,
            csrf: session.csrf_token,
            user_id: user.id,
            user_role: user.role,
            user_email: user.email,
            user_name: user.username,
            user_phone: user.phone,
            frozen: user.frozen,
            google_id: user.google_id,
            github_id: user.github_id,
            expires_at: session.expires_at.timestamp(),
        }
    }

    pub fn is_permanent(&self) -> bool {
        self.expires_at == NaiveDateTime::MAX.timestamp()
    }
}
