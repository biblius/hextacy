use crate::models::{session::Session, user::User};
use derive_new::new;
use infrastructure::http::response::Response;
use serde::Serialize;

/// Sent when the user completely authenticates
#[derive(Debug, Serialize, new)]
pub(super) struct AuthenticationSuccess {
    user: User,
    session: Session,
}
impl Response for AuthenticationSuccess {}

/// Sent when the user successfully authenticates with credentials and has 2FA enabled
#[derive(Debug, Serialize, new)]
pub(super) struct Prompt2FA<'a> {
    username: &'a str,
    token: &'a str,
    remember: bool,
}
impl<'a> Response for Prompt2FA<'a> {}

/// Sent when the user exceeds the maximum invalid login attempts
#[derive(Debug, Serialize, new)]
pub(super) struct FreezeAccount<'a> {
    email: &'a str,
    message: &'a str,
}
impl<'a> Response for FreezeAccount<'a> {}

/// Sent when a user registers for the very first time
#[derive(Debug, Serialize, new)]
pub(super) struct RegistrationSuccess<'a> {
    message: &'a str,
    username: &'a str,
    email: &'a str,
}
impl<'a> Response for RegistrationSuccess<'a> {}

/// Sent when a user successfully verifies their registration token
#[derive(Debug, Serialize, new)]
pub(super) struct TokenVerified<'a> {
    user_id: &'a str,
    message: &'a str,
}
impl<'a> Response for TokenVerified<'a> {}

#[derive(Debug, Serialize, new)]
pub(super) struct Logout<'a> {
    message: &'a str,
}
impl<'a> Response for Logout<'a> {}

#[derive(Debug, Serialize, new)]
pub(super) struct ChangedPassword<'a> {
    message: &'a str,
}
impl<'a> Response for ChangedPassword<'a> {}
