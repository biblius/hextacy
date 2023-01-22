use crate::{
    adapters::AdapterError,
    models::user::{SortOptions, User},
};

#[mockall::automock]
pub trait UserRepository {
    /// Create a user entry
    fn create(&self, email: &str, username: &str, password: &str) -> Result<User, AdapterError>;

    /// Get a user by their ID
    fn get_by_id(&self, id: &str) -> Result<User, AdapterError>;

    /// Get a user by their email
    fn get_by_email(&self, email: &str) -> Result<User, AdapterError>;

    /// Hash the given password with bcrypt and set the user's password field to the hash
    fn update_password(&self, id: &str, password: &str) -> Result<User, AdapterError>;

    /// Update the user's OTP secret to the given key
    fn update_otp_secret(&self, id: &str, secret: &str) -> Result<User, AdapterError>;

    /// Update the user's `email_verified_at` field to now
    fn update_email_verified_at(&self, id: &str) -> Result<User, AdapterError>;

    /// Set the user's frozen flag to true
    fn freeze(&self, id: &str) -> Result<User, AdapterError>;

    /// Return a vec of users constrained by the params
    fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort_by: Option<SortOptions>,
    ) -> Result<Vec<User>, AdapterError>;
}
