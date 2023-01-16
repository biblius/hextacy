use crate::helpers::validation::EMAIL_REGEX;
use derive_new::new;
use infrastructure::{store::repository::user::User, web::http::response::Response};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use validify::validify;

#[derive(Debug, Clone, Deserialize)]
#[validify]
/// Received on initial login
pub(super) struct Credentials {
    #[validate(regex = "EMAIL_REGEX")]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub remember: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[validify]
/// Received when registering
pub(super) struct RegistrationData {
    #[validate(regex = "EMAIL_REGEX")]
    pub email: String,
    #[modify(trim)]
    #[validate(length(min = 4))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[validify]
/// Received when resending reg token
pub(super) struct ResendRegToken {
    #[validate(regex = "EMAIL_REGEX")]
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[validify]
/// Received when verifying a one time password
pub(super) struct Otp {
    #[validate(length(equal = 6))]
    pub password: String,
    pub token: String,
    pub remember: bool,
}

#[derive(Debug, Deserialize)]
#[validify]
/// Received when updating a password
pub(super) struct ChangePassword {
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[validify]
/// Received when a user forgot their password
pub(super) struct ForgotPassword {
    #[validate(regex = "EMAIL_REGEX")]
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[validify]
/// Received when a user asks to reset their password via email
pub(super) struct ResetPassword {
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[validify]
/// Received when verifying registration token
pub(super) struct EmailToken {
    pub token: String,
}

#[derive(Debug, Deserialize)]
#[validify]
/// Received when verifying registration token
pub(super) struct ForgotPasswordVerify {
    #[validate(length(min = 8))]
    pub password: String,
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
