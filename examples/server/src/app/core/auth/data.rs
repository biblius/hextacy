use crate::{db::models::user::User, helpers::validation::EMAIL_REGEX};
use derive_new::new;
use hextacy::web::http::response::Response;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use validify::Validify;

#[derive(Debug, Clone, Deserialize, Validify)]
/// Received on initial login
pub struct Credentials {
    #[validate(regex(EMAIL_REGEX))]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub remember: bool,
}

#[derive(Debug, Clone, Deserialize, Validify)]
/// Received when registering
pub struct RegistrationData {
    #[validate(regex(EMAIL_REGEX))]
    pub email: String,
    #[modify(trim)]
    #[validate(length(min = 4))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validify)]
/// Received when resending reg token
pub struct ResendRegToken {
    #[validate(regex(EMAIL_REGEX))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validify)]
/// Received when verifying a one time password
pub struct Otp {
    #[validate(length(equal = 6))]
    pub password: String,
    pub token: String,
    pub remember: bool,
}

#[derive(Debug, Deserialize, Validify)]
/// Received when updating a password
pub struct ChangePassword {
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validify)]
/// Received when a user forgot their password
pub struct ForgotPassword {
    #[validate(regex(EMAIL_REGEX))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validify)]
/// Received when a user asks to reset their password via email
pub struct ResetPassword {
    pub token: String,
}

#[derive(Debug, Deserialize, Validify)]
/// Received when verifying registration token
pub struct EmailToken {
    pub token: String,
}

#[derive(Debug, Deserialize, Validify)]
/// Received when verifying registration token
pub struct ForgotPasswordVerify {
    #[validate(length(min = 8))]
    pub password: String,
    pub token: String,
}

#[derive(Debug, Deserialize)]
/// Received when verifying registration token
pub struct Logout {
    pub purge: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OAuthProvider {
    Google,
    Github,
}

#[derive(Debug, Clone, Deserialize, Validify)]
pub struct OAuthCodeExchange {
    #[modify(trim)]
    #[validate(length(min = 1))]
    pub code: String,
}

/*

RESPONSES

*/

/// Sent when the user successfully authenticates with credentials and has 2FA enabled
#[derive(Debug, Serialize, new)]
pub struct TwoFactorAuthResponse<'a> {
    username: &'a str,
    token: &'a str,
    remember: bool,
}
impl<'a> Response<'_> for TwoFactorAuthResponse<'a> {}

/// Sent when the user exceeds the maximum invalid login attempts
#[derive(Debug, Serialize, new)]
pub struct FreezeAccountResponse<'a> {
    email: &'a str,
    message: &'a str,
}
impl<'a> Response<'_> for FreezeAccountResponse<'a> {}

/// Sent when a user registers for the very first time
#[derive(Debug, Serialize, new)]
pub struct RegistrationStartResponse<'a> {
    message: &'a str,
    username: &'a str,
    email: &'a str,
}
impl<'a> Response<'_> for RegistrationStartResponse<'a> {}

/// Sent when the user completely authenticates
#[derive(Debug, Serialize, new)]
pub struct AuthenticationSuccessResponse {
    user: User,
}
impl Response<'_> for AuthenticationSuccessResponse {}
