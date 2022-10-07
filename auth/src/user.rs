use serde::{Deserialize, Serialize};
use storage::diesel::Queryable;

#[derive(Debug, Deserialize, Serialize, Queryable)]
pub struct User {
    id: String,
    email: String,
    username: String,
    #[serde(skip_serializing)]
    password: String,
    #[serde(skip_serializing)]
    otp_secret: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Queryable)]
pub struct Session {
    id: String,
    user_id: String,
    email: String,
    username: String,
    token: String,
    csrf: String,
}

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

#[derive(Debug, Deserialize)]
pub struct Otp<'a>(&'a str);

#[derive(Debug, Deserialize)]
pub struct Token<'a>(&'a str);
