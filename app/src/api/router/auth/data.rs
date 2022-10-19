use serde::Deserialize;
use std::fmt::Debug;

#[derive(Debug, Deserialize)]
/// Received on initial login
pub struct Credentials {
    email: String,
    password: String,
}

impl Credentials {
    pub fn data(&self) -> (&str, &str) {
        (&self.email, &self.password)
    }
}

#[derive(Debug, Deserialize)]
/// Received when verifying a one time password
pub struct Otp {
    password: String,
    token: String,
}

impl Otp {
    pub fn data(&self) -> (&str, &str) {
        (&self.password, &self.token)
    }
}

#[derive(Debug, Deserialize)]
/// Received when registering
pub(crate) struct RegistrationData {
    email: String,
    username: String,
}

impl RegistrationData {
    pub(crate) fn inner(&self) -> (&str, &str) {
        (&self.email, &self.username)
    }
}

#[derive(Debug, Deserialize)]
/// Received when setting a password for the first time
pub(crate) struct SetPassword {
    token: String,
    password: String,
}

impl SetPassword {
    pub(crate) fn inner(&self) -> (&str, &str) {
        (&self.token, &self.password)
    }
}

#[derive(Debug, Deserialize)]
/// Received when verifying registration token
pub(crate) struct EmailToken {
    token: String,
}

impl EmailToken {
    pub(crate) fn inner(&self) -> &str {
        &self.token
    }
}
