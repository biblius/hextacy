use infrastructure::http::response::Response;
use serde::Serialize;

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
