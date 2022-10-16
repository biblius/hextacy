use crate::error::Error;
use infrastructure::{
    config::{self},
    email::{self, lettre::SmtpTransport},
};
use std::sync::Arc;
use tracing::debug;

pub struct Email {
    client: Arc<SmtpTransport>,
}

impl Email {
    pub(crate) fn new(client: Arc<SmtpTransport>) -> Self {
        Self { client }
    }

    pub(super) fn send_registration_token(
        &self,
        token: &str,
        username: &str,
        email: &str,
    ) -> Result<(), Error> {
        debug!("Sending registration token to {}", email);
        let domain = config::env::get("DOMAIN").expect("DOMAIN must be set");
        let uri = format!("{domain}/auth/verify-registration-token?token={token}");
        let mail = email::generate_from_template(
            "registration_token",
            &[("username", username), ("registration_uri", &uri)],
        );
        email::send_email(
            None,
            username,
            email,
            "Finish registration",
            mail,
            &self.client,
        )
        .map_err(|e| e.into())
    }
}
