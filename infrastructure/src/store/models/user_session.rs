use crate::store::repository::{role::Role, session::Session, user::User};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

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
