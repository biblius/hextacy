use super::data::{
    ChangePassword, Credentials, EmailToken, Logout, Otp, RegistrationData, ResetPassword,
};
use crate::{error::Error, services::cache::CacheId};
use actix_web::HttpResponse;
use async_trait::async_trait;
use infrastructure::repository::{session::Session, user::User};
use serde::{de::DeserializeOwned, Serialize};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub(super) trait ServiceContract {
    async fn login(&self, credentails: Credentials) -> Result<HttpResponse, Error>;
    async fn verify_otp(&self, credentails: Otp) -> Result<HttpResponse, Error>;
    async fn start_registration(&self, data: RegistrationData) -> Result<HttpResponse, Error>;
    async fn verify_registration_token(&self, data: EmailToken) -> Result<HttpResponse, Error>;
    async fn set_otp_secret(&self, user_id: &str) -> Result<HttpResponse, Error>;
    async fn change_password(
        &self,
        session: Session,
        data: ChangePassword,
    ) -> Result<HttpResponse, Error>;
    async fn resend_registration_token(&self, user_id: &str) -> Result<HttpResponse, Error>;
    async fn reset_password(&self, data: ResetPassword) -> Result<HttpResponse, Error>;
    async fn logout(&self, session: Session, data: Logout) -> Result<HttpResponse, Error>;
    async fn purge_and_clear_sessions<'a>(
        &self,
        user_id: &str,
        skip: Option<&'a str>,
    ) -> Result<(), Error>;
    async fn session_response(&self, user: User, remember: bool) -> Result<HttpResponse, Error>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub(super) trait RepositoryContract {
    async fn create_user(&self, email: &str, username: &str, password: &str)
        -> Result<User, Error>;
    async fn get_user_by_id(&self, id: &str) -> Result<User, Error>;
    async fn get_user_by_email(&self, emal: &str) -> Result<User, Error>;
    async fn freeze_user(&self, id: &str) -> Result<User, Error>;
    async fn update_user_password(&self, id: &str, password: &str) -> Result<User, Error>;
    async fn update_email_verified_at(&self, id: &str) -> Result<User, Error>;
    async fn set_user_otp_secret(&self, id: &str, secret: &str) -> Result<User, Error>;
    async fn create_session(
        &self,
        user: &User,
        csrf_token: &str,
        permanent: bool,
    ) -> Result<Session, Error>;
    async fn expire_session(&self, session_id: &str) -> Result<Session, Error>;
    async fn purge_sessions<'a>(
        &self,
        user_id: &str,
        skip: Option<&'a str>,
    ) -> Result<Vec<Session>, Error>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub(super) trait CacheContract {
    async fn set_session(&self, csrf_token: &str, session: &Session) -> Result<(), Error>;
    async fn set_token<T: Serialize + Sync + Send + 'static>(
        &self,
        cache_id: CacheId,
        key: &str,
        value: &T,
        ex: Option<usize>,
    ) -> Result<(), Error>;
    async fn get_token<T: DeserializeOwned + Sync + Send + 'static>(
        &self,
        cache_id: CacheId,
        key: &str,
    ) -> Result<T, Error>;
    async fn delete_token(&self, cache_id: CacheId, key: &str) -> Result<(), Error>;
    async fn cache_login_attempt(&self, user_id: &str) -> Result<u8, Error>;
    async fn delete_login_attempts(&self, user_id: &str) -> Result<(), Error>;
    async fn cache_otp_throttle(&self, user_id: &str) -> Result<i64, Error>;
    async fn delete_otp_throttle(&self, user_id: &str) -> Result<(), Error>;
}

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub(super) trait EmailContract {
    async fn send_registration_token(
        &self,
        token: &str,
        username: &str,
        email: &str,
    ) -> Result<(), Error>;
    async fn alert_password_change(
        &self,
        username: &str,
        email: &str,
        token: &str,
    ) -> Result<(), Error>;
    async fn send_reset_password(
        &self,
        username: &str,
        email: &str,
        temp_pw: &str,
    ) -> Result<(), Error>;
    async fn send_freeze_account(
        &self,
        username: &str,
        email: &str,
        token: &str,
    ) -> Result<(), Error>;
}
