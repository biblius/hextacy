use super::role::Role;
use chrono::NaiveDateTime;
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable)]
#[diesel(table_name = user_sessions)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: String,
    pub role: Role,
    #[serde(skip_serializing)]
    pub password: String,
    #[serde(skip_serializing)]
    pub otp_secret: Option<String>,
    pub phone: Option<String>,
    pub google_id: Option<String>,
    pub github_id: Option<String>,
    pub frozen: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
