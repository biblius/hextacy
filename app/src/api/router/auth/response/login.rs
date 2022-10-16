use infrastructure::http::response::Response;
use serde::Serialize;

use crate::models::{session::Session, user::User};

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

#[derive(Debug, Serialize)]
pub struct FreezeAccount<'a> {
    user_id: &'a str,
    message: &'a str,
}

impl<'a> FreezeAccount<'a> {
    pub fn new(user_id: &'a str, message: &'a str) -> Self {
        Self { user_id, message }
    }
}

impl<'a> Response for FreezeAccount<'a> {}
