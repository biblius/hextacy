use crate::{config::cache::AuthCache, error::Error};
use alx_core::clients::oauth::{OAuthProvider, TokenResponse};
use async_trait::async_trait;
use storage::models::{oauth::OAuthMeta, session::Session, user::User};

#[cfg_attr(test, mockall::automock)]
#[async_trait(?Send)]
pub(super) trait RepositoryContract {
    async fn get_user_by_id(&self, id: &str) -> Result<User, Error>;
    async fn get_user_by_email(&self, email: &str) -> Result<User, Error>;

    async fn create_user(&self, email: &str, username: &str, pw: &str) -> Result<User, Error>;
    async fn update_user_email_verification(&self, id: &str) -> Result<User, Error>;
    async fn update_user_otp_secret(&self, id: &str, secret: &str) -> Result<User, Error>;
    async fn update_user_password(&self, id: &str, hashed_pw: &str) -> Result<User, Error>;
    async fn freeze_user(&self, id: &str) -> Result<User, Error>;

    async fn create_session<'a>(
        &self,
        user: &User,
        csrf: &str,
        expires: Option<i64>,
        access_token: Option<&'a str>,
        provider: Option<OAuthProvider>,
    ) -> Result<Session, Error>;
    async fn expire_session(&self, id: &str) -> Result<Session, Error>;
    async fn purge_sessions<'a>(
        &self,
        user_id: &str,
        skip: Option<&'a str>,
    ) -> Result<Vec<Session>, Error>;
    async fn update_session_access_tokens(
        &self,
        access_token: &str,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<Vec<Session>, Error>;

    async fn create_user_from_oauth(
        &self,
        account_id: &str,
        email: &str,
        username: &str,
        provider: OAuthProvider,
    ) -> Result<User, Error>;

    async fn update_user_provider_id(
        &self,
        user_id: &str,
        account_id: &str,
        provider: OAuthProvider,
    ) -> Result<User, Error>;

    async fn get_oauth_by_account_id(&self, account_id: &str) -> Result<OAuthMeta, Error>;

    async fn create_oauth<T>(
        &self,
        user_id: &str,
        account_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, Error>
    where
        T: TokenResponse + 'static;

    async fn update_oauth<T>(
        &self,
        user_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, Error>
    where
        T: TokenResponse + 'static;
}

#[cfg_attr(test, mockall::automock)]
pub(super) trait CacheContract {
    fn set_session(&self, session_id: &str, session: &Session) -> Result<(), Error>;

    // String tokens
    fn set_token(
        &self,
        cache_id: AuthCache,
        key: &str,
        value: &str,
        ex: Option<usize>,
    ) -> Result<(), Error>;
    fn get_token(&self, cache_id: AuthCache, key: &str) -> Result<String, Error>;
    fn delete_token(&self, cache_id: AuthCache, key: &str) -> Result<(), Error>;

    // Login
    fn cache_login_attempt(&self, user_id: &str) -> Result<u8, Error>;
    fn delete_login_attempts(&self, user_id: &str) -> Result<(), Error>;

    // Otp
    fn get_otp_throttle(&self, cache_id: AuthCache, user_id: &str) -> Result<i64, Error>;
    fn cache_otp_throttle(&self, user_id: &str) -> Result<i64, Error>;
    fn delete_otp_throttle(&self, user_id: &str) -> Result<(), Error>;

    // Email
    fn set_email_throttle(&self, user_id: &str) -> Result<(), Error>;
    fn get_email_throttle(&self, user_id: &str) -> Result<i64, Error>;
}

#[cfg_attr(test, mockall::automock)]
pub(super) trait EmailContract {
    fn send_registration_token(
        &self,
        token: &str,
        username: &str,
        email: &str,
    ) -> Result<(), Error>;
    fn alert_password_change(&self, username: &str, email: &str, token: &str) -> Result<(), Error>;
    fn send_reset_password(&self, username: &str, email: &str, temp_pw: &str) -> Result<(), Error>;
    fn send_forgot_password(&self, username: &str, email: &str, token: &str) -> Result<(), Error>;
    fn send_freeze_account(&self, username: &str, email: &str, token: &str) -> Result<(), Error>;
}
