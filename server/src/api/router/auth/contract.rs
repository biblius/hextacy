use super::data::{
    ChangePassword, Credentials, EmailToken, ForgotPassword, ForgotPasswordVerify, Logout, Otp,
    RegistrationData, ResendRegToken, ResetPassword,
};
use crate::{config::cache::AuthCache, error::Error};
use actix_web::HttpResponse;
use storage::models::{session::UserSession, user::User};

#[cfg_attr(test, mockall::automock)]
pub(super) trait ServiceContract {
    /// Verify the user's email and password and establish a session if they don't have 2FA. If the `remember`
    /// flag is true the session established will be permanent (applies for `verify_otp` as well).
    fn login(&self, credentails: Credentials) -> Result<HttpResponse, Error>;
    /// Verify the user's OTP and if successful establish a session.
    fn verify_otp(&self, credentails: Otp) -> Result<HttpResponse, Error>;
    /// Start the registration process and send a registration token via email.
    fn start_registration(&self, data: RegistrationData) -> Result<HttpResponse, Error>;
    /// Verify the registration token.
    fn verify_registration_token(&self, data: EmailToken) -> Result<HttpResponse, Error>;
    /// Resend a registration token in case the user's initial one expired.
    fn resend_registration_token(&self, data: ResendRegToken) -> Result<HttpResponse, Error>;
    /// Set the user's OTP secret and enable 2FA for the user. Send a QR code of the secret in the
    /// response. Requires an established session beforehand as it is not idempotent, meaning
    /// it will generate a new OTP secret every time this URL is called.
    fn set_otp_secret(&self, user_id: &str) -> Result<HttpResponse, Error>;
    /// Change the user's password, purge all their sessions and notify by email. Sets a
    /// temporary PW token in the cache. Works only with an established session.
    fn change_password(
        &self,
        session: UserSession,
        data: ChangePassword,
    ) -> Result<HttpResponse, Error>;
    /// Verify a token sent to a user via email when they request a forgotten password and change their
    /// password to the given one
    fn verify_forgot_password(&self, data: ForgotPasswordVerify) -> Result<HttpResponse, Error>;
    /// Reset the user's password and send it to their email. Works only if a temporary PW
    /// token is in the cache.
    fn reset_password(&self, data: ResetPassword) -> Result<HttpResponse, Error>;
    /// Reset the user's password
    fn forgot_password(&self, data: ForgotPassword) -> Result<HttpResponse, Error>;
    /// Log the user out, i.e. expire their current session and purge the rest if the user
    /// selected the purge option
    fn logout(&self, session: UserSession, data: Logout) -> Result<HttpResponse, Error>;
    /// Expire and remove from the cache all user sessions
    fn purge_sessions<'a>(&self, user_id: &str, skip: Option<&'a str>) -> Result<(), Error>;
    /// Generate a successful authentication response and set the necessary cookies and backend session data
    fn session_response(&self, user: User, remember: bool) -> Result<HttpResponse, Error>;
}

#[cfg_attr(test, mockall::automock)]
pub(super) trait CacheContract {
    fn set_session(&self, session_id: &str, session: &UserSession) -> Result<(), Error>;

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
