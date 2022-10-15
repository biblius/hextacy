use std::fmt::Debug;

use actix_web::{cookie::Cookie, HttpResponse, HttpResponseBuilder};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::models::{session::Session, user::User};

pub trait AuthenticationResponse
where
    Self: Sized + Serialize,
{
    fn to_response(self, cookies: Option<Vec<Cookie<'_>>>) -> HttpResponse {
        if let Some(cookies) = cookies {
            let mut response = HttpResponseBuilder::new(StatusCode::OK);
            for c in cookies {
                response.cookie(c);
            }
            response.json(self)
        } else {
            HttpResponseBuilder::new(StatusCode::OK).json(self)
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Prompt2FA {
    username: String,
    token: String,
}

impl Prompt2FA {
    pub fn new(username: String, token: String) -> Self {
        Self { username, token }
    }
}

impl AuthenticationResponse for Prompt2FA {}
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
impl AuthenticationResponse for AuthenticationSuccess {}

#[derive(Debug, Deserialize)]
pub struct Credentials<'a> {
    email: &'a str,
    password: &'a str,
}

impl<'a> Credentials<'a> {
    pub fn data(&self) -> (&'a str, &'a str) {
        (self.email, self.password)
    }
}

#[derive(Debug, Deserialize)]
pub struct Otp<'a> {
    password: &'a str,
    token: &'a str,
}

impl<'a> Otp<'a> {
    pub fn data(&self) -> (&'a str, &'a str) {
        (self.password, self.token)
    }
}
