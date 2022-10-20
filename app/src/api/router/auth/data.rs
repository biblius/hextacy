use infrastructure::validation::EMAIL_REGEX;
use serde::Deserialize;
use std::fmt::Debug;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
/// Received on initial login
pub(super) struct Credentials {
    #[validate(regex = "EMAIL_REGEX")]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
/// Received when registering
pub(super) struct RegistrationData {
    #[validate(regex = "EMAIL_REGEX")]
    pub email: String,
    #[validate(length(min = 2))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
/// Received when verifying a one time password
pub(super) struct Otp {
    #[validate(length(equal = 6))]
    pub password: String,
    #[validate(length(min = 1))]
    pub token: String,
}

#[derive(Debug, Deserialize, Validate)]
/// Received when updating a password
pub(super) struct SetPassword {
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
/// Received when verifying registration token
pub(super) struct EmailToken {
    #[validate(length(min = 1))]
    pub token: String,
}

#[derive(Debug, Deserialize)]
/// Received when verifying registration token
pub(super) struct Logout {
    pub purge: bool,
}
