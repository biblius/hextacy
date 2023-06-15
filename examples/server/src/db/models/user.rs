use super::role::Role;
use chrono::NaiveDateTime;
use diesel::{self, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct User {
    // Never use strings for UUIDs pls
    pub id: String,
    pub email: String,
    pub username: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub role: Role,
    pub phone: Option<String>,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    #[serde(skip_serializing)]
    pub otp_secret: Option<String>,
    #[serde(skip_serializing)]
    pub frozen: bool,
    #[serde(skip_serializing)]
    pub google_id: Option<String>,
    #[serde(skip_serializing)]
    pub github_id: Option<String>,
    #[serde(skip_serializing)]
    pub email_verified_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    pub fn __mock(
        id: String,
        email: &str,
        username: &str,
        password: Option<String>,
        with_otp: bool,
        verified: bool,
        frozen: bool,
    ) -> Self {
        Self {
            id,
            email: email.to_string(),
            username: username.to_string(),
            first_name: Some("Biblius".to_string()),
            last_name: Some("Khan".to_string()),
            role: Role::User,
            password,
            otp_secret: if with_otp {
                Some("ON2XAZLSL5ZWG4TFOQ======".to_string())
            } else {
                None
            },
            phone: None,
            google_id: None,
            github_id: None,
            frozen,
            email_verified_at: if verified {
                NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp(), 0)
            } else {
                None
            },
            created_at: NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp(), 0)
                .unwrap(),
            updated_at: NaiveDateTime::from_timestamp_opt(chrono::Utc::now().timestamp(), 0)
                .unwrap(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub enum SortOptions {
    #[serde(rename = "username")]
    UsernameAsc,
    #[serde(rename = "-username")]
    UsernameDesc,
    #[serde(rename = "email")]
    EmailAsc,
    #[serde(rename = "-email")]
    EmailDesc,
    #[serde(rename = "createdAt")]
    CreatedAtAsc,
    #[serde(rename = "-createdAt")]
    CreatedAtDesc,
}
