use super::role::Role;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;

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
    pub email_verified_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
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

#[async_trait]
pub trait UserRepository {
    type Error: Error;

    /// Create a user entry
    async fn create(
        &self,
        email: &str,
        username: &str,
        password: &str,
    ) -> Result<User, Self::Error>;

    /// Get a user by their ID
    async fn get_by_id(&self, id: &str) -> Result<User, Self::Error>;

    /// Get a user by their email
    async fn get_by_email(&self, email: &str) -> Result<User, Self::Error>;

    /// Hash the given password with bcrypt and set the user's password field to the hash
    async fn update_password(&self, id: &str, password: &str) -> Result<User, Self::Error>;

    /// Update the user's OTP secret to the given key
    async fn update_otp_secret(&self, id: &str, secret: &str) -> Result<User, Self::Error>;

    /// Update the user's `email_verified_at` field to now
    async fn update_email_verified_at(&self, id: &str) -> Result<User, Self::Error>;

    /// Set the user's frozen flag to true
    async fn freeze(&self, id: &str) -> Result<User, Self::Error>;

    /// Return a vec of users constrained by the params
    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort_by: Option<SortOptions>,
    ) -> Result<Vec<User>, Self::Error>;
}
