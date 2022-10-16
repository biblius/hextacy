use crate::{
    error::Error,
    models::{session::Session, user::User},
};
use infrastructure::storage::{postgres::Pg, DatabaseError};
use std::sync::Arc;
use tracing::debug;

pub(crate) struct Postgres {
    pool: Arc<Pg>,
}

impl Postgres {
    pub(crate) fn new(pool: Arc<Pg>) -> Self {
        Self { pool }
    }

    pub(super) async fn find_user_by_email(&self, email: &str) -> Result<User, Error> {
        User::get_by_email(email, &mut self.pool.connect()?)
    }

    pub(super) async fn create_session(&self, user: &User) -> Result<Session, Error> {
        debug!("Creating session for user: {}", &user.id);
        Session::create(user, &mut self.pool.connect()?)
    }

    pub(super) async fn freeze_user(&self, user_id: &str) -> Result<User, Error> {
        debug!("Freezing user with id: {}", user_id);
        let mut result = User::freeze(user_id, &mut self.pool.connect()?)?;
        result
            .pop()
            .ok_or_else(|| DatabaseError::DoesNotExist(format!("User ID: {}", user_id)).into())
    }

    pub(super) async fn create_user(&self, email: &str, username: &str) -> Result<User, Error> {
        debug!("Creating user with email: {}", email);
        User::create(email, username, &mut self.pool.connect()?)
    }

    pub(super) async fn update_user_password(
        &self,
        user_id: &str,
        password: &str,
    ) -> Result<User, Error> {
        debug!("Updating password for user: {}", user_id);
        let mut result = User::update_password(user_id, password, &mut self.pool.connect()?)?;
        result
            .pop()
            .ok_or_else(|| DatabaseError::DoesNotExist(format!("User ID: {}", user_id)).into())
    }

    pub(super) async fn update_email_verified_at(&self, user_id: &str) -> Result<User, Error> {
        debug!("Updating verification status for: {}", user_id);
        let mut result = User::update_email_verified_at(user_id, &mut self.pool.connect()?)?;
        result
            .pop()
            .ok_or_else(|| DatabaseError::DoesNotExist(format!("User ID: {}", user_id)).into())
    }
}
