use crate::error::Error;
use infrastructure::{
    config::{self},
    email::{self, lettre::SmtpTransport},
};
use std::sync::Arc;
use tracing::debug;

pub(in super::super) struct Email {
    client: Arc<SmtpTransport>,
}

impl Email {
    pub(in super::super) fn new(client: Arc<SmtpTransport>) -> Self {
        Self { client }
    }

    pub(in super::super) fn send_registration_token(
        &self,
        token: &str,
        username: &str,
        email: &str,
    ) -> Result<(), Error> {
        debug!("Sending registration token email to {email}");
        let domain = config::env::get("DOMAIN").expect("DOMAIN must be set");
        let uri = format!("{domain}/auth/verify-registration-token?token={token}");
        let mail = email::from_template(
            "registration_token",
            &[("username", username), ("registration_uri", &uri)],
        );
        email::send(
            None,
            username,
            email,
            "Finish registration",
            mail,
            &self.client,
        )
        .map_err(Error::new)
    }

    pub(in super::super) fn send_password_change(
        &self,
        token: &str,
        username: &str,
        email: &str,
    ) -> Result<(), Error> {
        debug!("Sending change password email to {email}");
        let domain = config::env::get("DOMAIN").expect("DOMAIN must be set");
        let uri = format!("{domain}/auth/verify-registration-token?token={token}");
        let mail = email::from_template(
            "registration_token",
            &[("username", username), ("change_password_uri", &uri)],
        );
        email::send(None, username, email, "Password change", mail, &self.client)
            .map_err(Error::new)
    }
}
