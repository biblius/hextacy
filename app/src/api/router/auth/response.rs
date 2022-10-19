use infrastructure::http::response::Response;
use serde::Serialize;

use crate::models::{session::Session, user::User};

/// Sent when the user completely authenticates
#[derive(Debug, Serialize)]
pub struct AuthenticationSuccess {
    user: User,
    session: Session,
}

impl AuthenticationSuccess {
    pub fn new(user: User, session: Session) -> Self {
        Self { user, session }
    }
}

impl Response for AuthenticationSuccess {}

/// Sent when the user successfully authenticates with credentials and has 2FA enabled
#[derive(Debug, Serialize)]
pub struct Prompt2FA<'a> {
    username: &'a str,
    token: &'a str,
}

impl<'a> Prompt2FA<'a> {
    pub fn new(username: &'a str, token: &'a str) -> Self {
        Self { username, token }
    }
}

impl<'a> Response for Prompt2FA<'a> {}

/// Sent when the user exceeds the maximum invalid login attempts
#[derive(Debug, Serialize)]
pub struct FreezeAccount<'a> {
    email: &'a str,
    message: &'a str,
}

impl<'a> FreezeAccount<'a> {
    pub fn new(email: &'a str, message: &'a str) -> Self {
        Self { email, message }
    }
}

impl<'a> Response for FreezeAccount<'a> {}

/// Sent when a user registers for the very first time
#[derive(Debug, Serialize)]
pub struct RegistrationSuccess<'a> {
    message: &'a str,
    username: &'a str,
    email: &'a str,
}

impl<'a> RegistrationSuccess<'a> {
    pub fn new(message: &'a str, username: &'a str, email: &'a str) -> Self {
        Self {
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
    pub message: &'a str,
}

impl<'a> TokenVerified<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }
}

impl<'a> Response for TokenVerified<'a> {}
