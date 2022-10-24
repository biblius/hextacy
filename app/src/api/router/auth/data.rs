use derive_new::new;
use infrastructure::{
    repository::{session::Session, user::User},
    web::{http::response::Response, validation::EMAIL_REGEX},
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Validate)]
/// Received on initial login
pub(super) struct Credentials {
    #[validate(regex = "EMAIL_REGEX")]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub remember: bool,
}

#[derive(Debug, Clone, Deserialize, Validate)]
/// Received when registering
pub(super) struct RegistrationData {
    #[validate(regex = "EMAIL_REGEX")]
    pub email: String,
    #[validate(length(min = 2))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
/// Received when verifying a one time password
pub(super) struct Otp {
    #[validate(length(min = 1))]
    pub token: String,
    #[validate(length(equal = 6))]
    pub password: String,
    pub remember: bool,
}

#[derive(Debug, Deserialize, Validate)]
/// Received when updating a password
pub(super) struct ChangePassword {
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
/// Received when a user asks to reset their password via email
pub(super) struct ResetPassword {
    pub token: String,
}

#[derive(Debug, Deserialize, Validate)]
/// Received when verifying registration token
pub(super) struct EmailToken {
    #[validate(length(min = 1))]
    pub token: String,
}

#[derive(Debug, Deserialize)]
/// Received when verifying registration token
pub(super) struct Logout {
    pub purge: bool,
}

/*

RESPONSES

*/

/// Sent when the user completely authenticates
#[derive(Debug, Serialize, new)]
pub(super) struct AuthenticationSuccessResponse {
    user: User,
    session: Session,
}
impl Response for AuthenticationSuccessResponse {}

/// Sent when the user successfully authenticates with credentials and has 2FA enabled
#[derive(Debug, Serialize, new)]
pub(super) struct TwoFactorAuthResponse<'a> {
    username: &'a str,
    token: &'a str,
    remember: bool,
}
impl<'a> Response for TwoFactorAuthResponse<'a> {}

/// Sent when the user exceeds the maximum invalid login attempts
#[derive(Debug, Serialize, new)]
pub(super) struct FreezeAccountResponse<'a> {
    email: &'a str,
    message: &'a str,
}
impl<'a> Response for FreezeAccountResponse<'a> {}

/// Sent when a user registers for the very first time
#[derive(Debug, Serialize, new)]
pub(super) struct RegistrationStartResponse<'a> {
    message: &'a str,
    username: &'a str,
    email: &'a str,
}
impl<'a> Response for RegistrationStartResponse<'a> {}

/// Sent when a user successfully verifies their registration token
#[derive(Debug, Serialize, new)]
pub(super) struct RegistrationSuccessResponse<'a> {
    user_id: &'a str,
    message: &'a str,
}
impl<'a> Response for RegistrationSuccessResponse<'a> {}

/// Sent when a user successfully logs out
#[derive(Debug, Serialize, new)]
pub(super) struct LogoutResponse<'a> {
    message: &'a str,
}
impl<'a> Response for LogoutResponse<'a> {}

/// Sent when a user successfully changes their password
#[derive(Debug, Serialize, new)]
pub(super) struct ChangePasswordResponse<'a> {
    message: &'a str,
}
impl<'a> Response for ChangePasswordResponse<'a> {}

/// Sent when a user requests a password reset
#[derive(Debug, Serialize, new)]
pub(super) struct ResetPasswordResponse<'a> {
    message: &'a str,
}
impl<'a> Response for ResetPasswordResponse<'a> {}
