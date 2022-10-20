use crate::models::{session::Session, user::User};
use infrastructure::http::response::Response;
use serde::Serialize;

/// Sent when the user completely authenticates
#[derive(Debug, Serialize)]
pub struct AuthenticationSuccess<'a> {
    code: &'a str,
    user: User,
    session: Session,
}

impl<'a> AuthenticationSuccess<'a> {
    pub fn new(user: User, session: Session) -> Self {
        Self {
            code: "AUTHENTICATION_SUCCESS",
            user,
            session,
        }
    }
}

impl<'a> Response for AuthenticationSuccess<'a> {}

/// Sent when the user successfully authenticates with credentials and has 2FA enabled
#[derive(Debug, Serialize)]
pub struct Prompt2FA<'a> {
    code: &'a str,
    username: &'a str,
    token: &'a str,
}

impl<'a> Prompt2FA<'a> {
    pub fn new(username: &'a str, token: &'a str) -> Self {
        Self {
            code: "2FA_REQUIRED",
            username,
            token,
        }
    }
}

impl<'a> Response for Prompt2FA<'a> {}

/// Sent when the user exceeds the maximum invalid login attempts
#[derive(Debug, Serialize)]
pub struct FreezeAccount<'a> {
    code: &'a str,
    email: &'a str,
    message: &'a str,
}

impl<'a> FreezeAccount<'a> {
    pub fn new(email: &'a str, message: &'a str) -> Self {
        Self {
            code: "ACCOUNT_FROZEN",
            email,
            message,
        }
    }
}

impl<'a> Response for FreezeAccount<'a> {}

/// Sent when a user registers for the very first time
#[derive(Debug, Serialize)]
pub struct RegistrationSuccess<'a> {
    code: &'a str,
    message: &'a str,
    username: &'a str,
    email: &'a str,
}

impl<'a> RegistrationSuccess<'a> {
    pub fn new(message: &'a str, username: &'a str, email: &'a str) -> Self {
        Self {
            code: "REGISTRATION_SUCCESS",
            message,
            username,
            email,
        }
    }
}

impl<'a> Response for RegistrationSuccess<'a> {}

/// Sent when a user successfully verifies their registration token
#[derive(Debug, Serialize)]
pub struct TokenVerified<'a> {
    pub code: &'a str,
    pub message: &'a str,
}

impl<'a> TokenVerified<'a> {
    pub fn new(message: &'a str) -> Self {
        Self {
            code: "REGISTRATION_TOKEN_SUCCESS",
            message,
        }
    }
}

impl<'a> Response for TokenVerified<'a> {}

/// Sent when a user's temporary password expires and they need a new PW token
/// to set their password
#[derive(Debug, Serialize)]
pub struct ResendPWToken<'a> {
    pub code: &'a str,
    pub message: &'a str,
    pub token: &'a str,
}

impl<'a> ResendPWToken<'a> {
    pub fn new(message: &'a str, token: &'a str) -> Self {
        Self {
            code: "PW_TOKEN",
            message,
            token,
        }
    }
}

impl<'a> Response for ResendPWToken<'a> {}
