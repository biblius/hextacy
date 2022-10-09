use reqwest::StatusCode;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
pub struct Authentication<'a> {
    #[serde(alias = "type", rename = "type")]
    pub auth_type: AuthenticationType,
    pub data: AuthenticationData<'a>,
}

#[derive(Debug, Deserialize)]
pub enum AuthenticationType {
    #[serde(rename(deserialize = "credentials"))]
    Credentials,
    #[serde(rename(deserialize = "otp"))]
    OTP,
    #[serde(rename(deserialize = "token"))]
    Token,
}
#[derive(Debug, Deserialize)]
#[serde(untagged, bound(deserialize = "'de: 'a"))]
pub enum AuthenticationData<'a> {
    Credentials(Credentials<'a>),
    OTP(Otp<'a>),
    Token(Token<'a>),
}

#[derive(Debug, Deserialize)]
pub struct Credentials<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

impl<'a> Credentials<'a> {
    pub fn data(&self) -> (&'a str, &'a str) {
        (self.email, self.password)
    }
}

#[derive(Debug, Deserialize)]
pub struct Otp<'a>(&'a str);

impl<'a> Otp<'a> {
    pub fn password(&self) -> &'a str {
        self.0
    }
}

#[derive(Debug, Deserialize)]
pub struct Token<'a>(&'a str);

impl<'a> Token<'a> {
    pub fn token(&self) -> &'a str {
        self.0
    }
}

#[derive(Debug, Error)]
pub enum AuthenticationError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Invalid OTP")]
    InvalidOTP,
    #[error("Invalid role")]
    InvalidRole,
}

impl AuthenticationError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::InvalidToken => StatusCode::UNAUTHORIZED,
            Self::InvalidOTP => StatusCode::UNAUTHORIZED,
            Self::InvalidRole => StatusCode::UNPROCESSABLE_ENTITY,
        }
    }
}
