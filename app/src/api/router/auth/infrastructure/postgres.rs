use crate::{
    error::Error,
    models::{session::Session, user::User},
};
use infrastructure::storage::{postgres::Pg, DatabaseError};
use std::sync::Arc;
use tracing::debug;

pub(in super::super) struct Postgres {
    pool: Arc<Pg>,
}

impl Postgres {
    pub(in super::super) fn new(pool: Arc<Pg>) -> Self {
        Self { pool }
    }

    /// Gets a user by their id
    pub(in super::super) async fn get_user_by_id(&self, id: &str) -> Result<User, Error> {
        debug!("Getting user with ID {}", id);
        User::get_by_id(id, &mut self.pool.connect()?)
    }

    /// Gets a user by their email
    pub(in super::super) async fn get_user_by_email(&self, email: &str) -> Result<User, Error> {
        debug!("Getting user with email {}", email);
        User::get_by_email(email, &mut self.pool.connect()?)
    }

    /// Creates session for given user
    pub(in super::super) async fn create_session(
        &self,
        user: &User,
        csrf_token: &str,
    ) -> Result<Session, Error> {
        debug!("Creating session for user: {}", &user.id);
        Session::create(user, csrf_token, &mut self.pool.connect()?)
    }

    /// Marks the user's account as frozen
    pub(in super::super) async fn freeze_user(&self, user_id: &str) -> Result<User, Error> {
        debug!("Freezing user with id: {}", user_id);
        User::freeze(user_id, &mut self.pool.connect()?)?
            .pop()
            .ok_or_else(|| DatabaseError::DoesNotExist(format!("User ID: {user_id}")).into())
    }

    /// Creates a new unauthenticated user
    pub(in super::super) async fn create_user(
        &self,
        email: &str,
        username: &str,
    ) -> Result<User, Error> {
        debug!("Creating user with email: {}", email);
        User::create(email, username, &mut self.pool.connect()?)
    }

    /// Updates the user's password field
    pub(in super::super) async fn update_user_password(
        &self,
        user_id: &str,
        password: &str,
    ) -> Result<User, Error> {
        debug!("Updating password for user: {}", user_id);
        User::update_password(user_id, password, &mut self.pool.connect()?)?
            .pop()
            .ok_or_else(|| DatabaseError::DoesNotExist(format!("User ID: {user_id}")).into())
    }

    /// Updates the user's email_verified_at field upon successfully verifying their registration token
    pub(in super::super) async fn update_email_verified_at(
        &self,
        user_id: &str,
    ) -> Result<User, Error> {
        debug!("Updating verification status for: {}", user_id);
        User::update_email_verified_at(user_id, &mut self.pool.connect()?)?
            .pop()
            .ok_or_else(|| DatabaseError::DoesNotExist(format!("User ID: {user_id}")).into())
    }

    /// Generates a random OTP secret and stores it to the user
    pub(in super::super) async fn set_user_otp_secret(
        &self,
        user_id: &str,
        secret: &str,
    ) -> Result<User, Error> {
        debug!("Setting OTP secret for: {}", user_id);
        User::update_otp_secret(user_id, secret, &mut self.pool.connect()?)?
            .pop()
            .ok_or_else(|| DatabaseError::DoesNotExist(format!("User ID: {user_id}")).into())
    }
}
