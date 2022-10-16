use serde::Deserialize;
use std::fmt::Debug;

#[derive(Debug, Deserialize)]
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
pub struct Otp {
    password: String,
    token: String,
}

impl Otp {
    pub fn data(&self) -> (&str, &str) {
        (&self.password, &self.token)
    }
}
