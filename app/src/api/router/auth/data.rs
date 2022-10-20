use serde::Deserialize;
use std::fmt::Debug;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
/// Received on initial login
pub(super) struct Credentials {
    #[validate(email)]
    email: String,
    #[validate(length(min = 1))]
    password: String,
}

impl Credentials {
    pub(super) fn data(&self) -> (&str, &str) {
        (&self.email, &self.password)
    }
}

#[derive(Debug, Deserialize, Validate)]
/// Received when verifying a one time password
pub(super) struct Otp {
    #[validate(length(min = 6))]
    password: String,
    #[validate(length(min = 1))]
    token: String,
}

impl Otp {
    pub(super) fn data(&self) -> (&str, &str) {
        (&self.password, &self.token)
    }
}

#[derive(Debug, Deserialize, Validate)]
/// Received when registering
pub(super) struct RegistrationData {
    #[validate(email)]
    email: String,
    #[validate(length(min = 2))]
    username: String,
}

impl RegistrationData {
    pub(super) fn inner(&self) -> (&str, &str) {
        (&self.email, &self.username)
    }
}

#[derive(Debug, Deserialize, Validate)]
/// Received when setting a password for the first time
pub(super) struct SetPassword {
    #[validate(length(min = 1))]
    token: String,
    #[validate(length(min = 8))]
    password: String,
}

impl SetPassword {
    pub(super) fn inner(&self) -> (&str, &str) {
        (&self.token, &self.password)
    }
}

#[derive(Debug, Deserialize, Validate)]
/// Received when verifying registration token
pub(super) struct EmailToken {
    #[validate(length(min = 1))]
    token: String,
}

impl EmailToken {
    pub(super) fn inner(&self) -> &str {
        &self.token
    }
}
