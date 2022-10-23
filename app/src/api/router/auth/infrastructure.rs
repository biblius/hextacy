use super::contract::{CacheContract, EmailContract, RepositoryContract};
use crate::services::cache::Cache as Cacher;
use crate::{error::Error, services::cache::CacheId};
use async_trait::async_trait;
use infrastructure::clients::email;
use infrastructure::clients::email::lettre::SmtpTransport;
use infrastructure::config;
use infrastructure::repository::session::{Session, SessionRepository};
use infrastructure::repository::user::UserRepository;
use infrastructure::{adapters::postgres::PgAdapterError, repository::user::User};
use infrastructure::{
    clients::redis::{Commands, Redis},
    config::constants::{SESSION_CACHE_DURATION_SECONDS, WRONG_PASSWORD_CACHE_DURATION},
};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tracing::debug;

#[derive(Debug)]
pub(super) struct Repository<UR, SR>
where
    UR: UserRepository,
    SR: SessionRepository,
{
    pub user_repo: UR,
    pub session_repo: SR,
}

#[async_trait]
impl<UR, SR> RepositoryContract for Repository<UR, SR>
where
    UR: UserRepository<Error = PgAdapterError> + Send + Sync,
    SR: SessionRepository<Error = PgAdapterError> + Send + Sync,
{
    /// Creates a new user
    async fn create_user(
        &self,
        email: &str,
        username: &str,
        password: &str,
    ) -> Result<User, Error> {
        debug!("Creating user with email: {}", email);
        self.user_repo
            .create(email, username, password)
            .await
            .map_err(Error::new)
    }

    /// Gets a user by their id
    async fn get_user_by_id(&self, id: &str) -> Result<User, Error> {
        debug!("Getting user with ID {}", id);
        self.user_repo.get_by_id(id).await.map_err(Error::new)
    }

    /// Gets a user by their email
    async fn get_user_by_email(&self, email: &str) -> Result<User, Error> {
        debug!("Getting user with email {}", email);
        self.user_repo.get_by_email(email).await.map_err(Error::new)
    }

    /// Marks the user's account as frozen
    async fn freeze_user(&self, user_id: &str) -> Result<User, Error> {
        debug!("Freezing user with id: {user_id}");
        self.user_repo.freeze(user_id).await.map_err(Error::new)
    }

    /// Updates the user's password field
    async fn update_user_password(&self, user_id: &str, pw_hash: &str) -> Result<User, Error> {
        debug!("Updating password for user: {user_id}");
        self.user_repo
            .update_password(user_id, pw_hash)
            .await
            .map_err(Error::new)
    }

    /// Updates the user's email_verified_at field upon successfully verifying their registration token
    async fn update_email_verified_at(&self, user_id: &str) -> Result<User, Error> {
        debug!("Updating verification status for: {user_id}");
        self.user_repo
            .update_email_verified_at(user_id)
            .await
            .map_err(Error::new)
    }

    /// Generates a random OTP secret and stores it to the user
    async fn set_user_otp_secret(&self, user_id: &str, secret: &str) -> Result<User, Error> {
        debug!("Setting OTP secret for: {user_id}");
        self.user_repo
            .update_otp_secret(user_id, secret)
            .await
            .map_err(Error::new)
    }

    /// Creates session for given user
    async fn create_session(
        &self,
        user: &User,
        csrf_token: &str,
        permanent: bool,
    ) -> Result<Session, Error> {
        debug!("Creating session for user: {}", &user.id);
        self.session_repo
            .create(user, csrf_token, permanent)
            .await
            .map_err(Error::new)
    }

    /// Expires user session
    async fn expire_session(&self, session_id: &str) -> Result<Session, Error> {
        debug!("Expiring session for: {session_id}");
        self.session_repo
            .expire(session_id)
            .await
            .map_err(Error::new)
    }

    /// Expires all user sessions
    async fn purge_sessions<'a>(
        &self,
        user_id: &str,
        skip: Option<&'a str>,
    ) -> Result<Vec<Session>, Error> {
        debug!("Purging all sessions for: {user_id}");
        self.session_repo
            .purge(user_id, skip)
            .await
            .map_err(|_| PgAdapterError::DoesNotExist(format!("User ID: {user_id}")).into())
    }
}

pub(super) struct Cache {
    pub client: Arc<Redis>,
}

#[async_trait]
impl CacheContract for Cache {
    /// Sessions get cached behind the user's csrf token.
    async fn set_session(&self, csrf_token: &str, session: &Session) -> Result<(), Error> {
        let mut connection = self.client.connect()?;
        Cacher::set(
            CacheId::Session,
            csrf_token,
            session,
            Some(SESSION_CACHE_DURATION_SECONDS),
            &mut connection,
        )
        .map_err(Error::new)
    }

    /// Sets a token as a key to the provided value in the cache
    async fn set_token<T: Serialize + Sync + Send>(
        &self,
        cache_id: CacheId,
        token: &str,
        value: &T,
        ex: Option<usize>,
    ) -> Result<(), Error> {
        let mut connection = self.client.connect()?;
        Cacher::set(cache_id, token, value, ex, &mut connection).map_err(Error::new)
    }

    /// Gets a value from the cache stored under the token
    async fn get_token<T: DeserializeOwned + Sync + Send>(
        &self,
        cache_id: CacheId,
        token: &str,
    ) -> Result<T, Error> {
        let mut connection = self.client.connect()?;
        Cacher::get(cache_id, token, &mut connection).map_err(Error::new)
    }

    /// Deletes the value in the cache stored under the token
    async fn delete_token(&self, cache_id: CacheId, token: &str) -> Result<(), Error> {
        let mut connection = self.client.connect()?;
        Cacher::delete(cache_id, token, &mut connection).map_err(Error::new)
    }

    /// Caches the number of login attempts using the user ID as the key. If the attempts do not exist they
    /// will be created, otherwise they will be incremented.
    async fn cache_login_attempt(&self, user_id: &str) -> Result<u8, Error> {
        let mut connection = self.client.connect()?;
        let key = Cacher::prefix_id(CacheId::LoginAttempts, &user_id);
        match connection.incr::<&str, u8, u8>(&key, 1) {
            Ok(c) => Ok(c),
            Err(_) => connection
                .set_ex::<String, u8, u8>(key, 1, WRONG_PASSWORD_CACHE_DURATION)
                .map_err(Error::new),
        }
    }

    /// Removes the user's login attempts from the cache
    async fn delete_login_attempts(&self, user_id: &str) -> Result<(), Error> {
        debug!("Deleting login attempts for: {}", &user_id);
        let mut connection = self.client.connect()?;
        Cacher::delete(CacheId::LoginAttempts, user_id, &mut connection).map_err(Error::new)
    }
}

pub(super) struct Email {
    pub client: Arc<SmtpTransport>,
}

#[async_trait]
impl EmailContract for Email {
    async fn send_registration_token(
        &self,
        token: &str,
        username: &str,
        email: &str,
    ) -> Result<(), Error> {
        debug!("Sending registration token email to {email}");
        let domain = config::env::get("DOMAIN").expect("DOMAIN must be set");
        let uri = format!("{domain}/auth/verify-registration-token?token={token}");
        let mail = email::from_template(
            "registration_token",
            &[("username", username), ("registration_uri", &uri)],
        );
        email::send(
            None,
            username,
            email,
            "Finish registration",
            mail,
            &self.client,
        )
        .map_err(Error::new)
    }

    async fn send_reset_password(
        &self,
        username: &str,
        email: &str,
        temp_pw: &str,
    ) -> Result<(), Error> {
        debug!("Sending reset password email to {email}");
        let mail = email::from_template(
            "reset_password",
            &[("username", username), ("temp_password", temp_pw)],
        );
        email::send(None, username, email, "Reset password", mail, &self.client).map_err(Error::new)
    }

    async fn alert_password_change(
        &self,
        token: &str,
        username: &str,
        email: &str,
    ) -> Result<(), Error> {
        debug!("Sending change password email alert to {email}");
        let domain = config::env::get("DOMAIN").expect("DOMAIN must be set");
        let uri = format!("{domain}/auth/reset-password?token={token}");
        let mail = email::from_template(
            "change_password",
            &[("username", username), ("change_password_uri", &uri)],
        );
        email::send(None, username, email, "Password change", mail, &self.client)
            .map_err(Error::new)
    }
}
