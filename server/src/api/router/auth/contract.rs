use storage::models::session::Session;

use crate::{config::cache::AuthCache, error::Error};

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
