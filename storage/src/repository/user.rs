use crate::{
    adapters::AdapterError,
    models::user::{SortOptions, User},
};
use alx_clients::oauth::OAuthProvider;
use async_trait::async_trait;

#[async_trait]
pub trait UserRepository<C> {
    /// Create a user entry
    async fn create(
        conn: &mut C,
        email: &str,
        username: &str,
        password: &str,
    ) -> Result<User, AdapterError>;

    async fn create_from_oauth(
        conn: &mut C,
        account_id: &str,
        email: &str,
        username: &str,
        provider: OAuthProvider,
    ) -> Result<User, AdapterError>;

    /// Get a user by their ID
    async fn get_by_id(conn: &mut C, id: &str) -> Result<User, AdapterError>;

    /// Get a user by their oauth ID
    async fn get_by_oauth_id(
        conn: &mut C,
        id: &str,
        provider: OAuthProvider,
    ) -> Result<User, AdapterError>;

    /// Get a user by their email
    async fn get_by_email(conn: &mut C, email: &str) -> Result<User, AdapterError>;

    /// Hash the given password with bcrypt and set the user's password field to the hash
    async fn update_password(conn: &mut C, id: &str, password: &str) -> Result<User, AdapterError>;

    /// Update the user's OTP secret to the given key
    async fn update_otp_secret(conn: &mut C, id: &str, secret: &str) -> Result<User, AdapterError>;

    /// Update the user's `email_verified_at` field to now
    async fn update_email_verified_at(conn: &mut C, id: &str) -> Result<User, AdapterError>;

    /// Update one of the user's oauth IDs
    async fn update_oauth_id(
        conn: &mut C,
        id: &str,
        oauth_id: &str,
        provider: OAuthProvider,
    ) -> Result<User, AdapterError>;

    /// Set the user's frozen flag to true
    async fn freeze(conn: &mut C, id: &str) -> Result<User, AdapterError>;

    /// Return a vec of users constrained by the params
    async fn get_paginated(
        conn: &mut C,
        page: u16,
        per_page: u16,
        sort_by: Option<SortOptions>,
    ) -> Result<Vec<User>, AdapterError>;
}
