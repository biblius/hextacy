use crate::storage::repository::role::Role;
use chrono::NaiveDateTime;
use diesel::Queryable;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
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
    #[serde(skip_serializing)]
    pub email_verified_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    /// Checks if the user is suspended and if their email is verified.
    pub fn check_valid(&self) -> bool {
        !self.frozen && self.email_verified_at.is_some()
    }

    pub fn __mock(
        id: String,
        email: &str,
        username: &str,
        password: String,
        with_otp: bool,
        verified: bool,
        frozen: bool,
    ) -> Self {
        Self {
            id,
            email: email.to_string(),
            username: username.to_string(),
            role: Role::User,
            password,
            otp_secret: if with_otp {
                Some(data_encoding::BASE32.encode(b"super_scret"))
            } else {
                None
            },
            phone: None,
            google_id: None,
            github_id: None,
            frozen,
            email_verified_at: if verified {
                Some(NaiveDateTime::from_timestamp(
                    chrono::Utc::now().timestamp(),
                    0,
                ))
            } else {
                None
            },
            created_at: NaiveDateTime::from_timestamp(chrono::Utc::now().timestamp(), 0),
            updated_at: NaiveDateTime::from_timestamp(chrono::Utc::now().timestamp(), 0),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy)]
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
