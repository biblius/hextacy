use super::data::{
    ChangePassword, Credentials, EmailToken, ForgotPassword, ForgotPasswordVerify, Logout, Otp,
    RegistrationData, ResendRegToken, ResetPassword,
};
use crate::error::Error;
use actix_web::HttpResponse;
use async_trait::async_trait;
use storage::models::{session::Session, user::User};

#[cfg_attr(test, mockall::automock)]
#[async_trait(?Send)]
pub(super) trait ServiceContract {
    /// Verify the user's email and password and establish a session if they don't have 2FA. If the `remember`
    /// flag is true the session established will be permanent (applies for `verify_otp` as well).
    async fn login(&self, credentails: Credentials) -> Result<HttpResponse, Error>;

    /// Verify the user's OTP and if successful establish a session.
    async fn verify_otp(&self, credentails: Otp) -> Result<HttpResponse, Error>;

    /// Start the registration process and send a registration token via email.
    async fn start_registration(&self, data: RegistrationData) -> Result<HttpResponse, Error>;

    /// Verify the registration token.
    async fn verify_registration_token(&self, data: EmailToken) -> Result<HttpResponse, Error>;

    /// Resend a registration token in case the user's initial one expired.
    async fn resend_registration_token(&self, data: ResendRegToken) -> Result<HttpResponse, Error>;

    /// Set the user's OTP secret and enable 2FA for the user. Send a QR code of the secret in the
    /// response. Requires an established session beforehand as it is not idempotent, meaning
    /// it will generate a new OTP secret every time this URL is called.
    async fn set_otp_secret(&self, session: Session) -> Result<HttpResponse, Error>;

    /// Change the user's password, purge all their sessions and notify by email. Sets a
    /// temporary PW token in the cache. Works only with an established session.
    async fn change_password(
        &self,
        session: Session,
        data: ChangePassword,
    ) -> Result<HttpResponse, Error>;

    /// Verify a token sent to a user via email when they request a forgotten password and change their
    /// password to the given one
    async fn verify_forgot_password(
        &self,
        data: ForgotPasswordVerify,
    ) -> Result<HttpResponse, Error>;

    /// Reset the user's password and send it to their email. Works only if a temporary PW
    /// token is in the cache.
    async fn reset_password(&self, data: ResetPassword) -> Result<HttpResponse, Error>;

    /// Reset the user's password
    async fn forgot_password(&self, data: ForgotPassword) -> Result<HttpResponse, Error>;

    /// Log the user out, i.e. expire their current session and purge the rest if the user
    /// selected the purge option
    async fn logout(&self, session: Session, data: Logout) -> Result<HttpResponse, Error>;

    /// Expire and remove from the cache all user sessions
    async fn purge_sessions<'a>(&self, user_id: &str, skip: Option<&'a str>) -> Result<(), Error>;

    /// Generate a successful authentication response and set the necessary cookies and backend session data
    async fn establish_session(&self, user: User, remember: bool) -> Result<HttpResponse, Error>;
}
