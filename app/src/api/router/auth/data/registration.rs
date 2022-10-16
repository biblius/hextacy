use serde::Deserialize;

#[derive(Debug, Deserialize)]
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
pub(crate) struct EmailToken {
    token: String,
}

impl EmailToken {
    pub(crate) fn inner(&self) -> &str {
        &self.token
    }
}
