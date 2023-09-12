use crate::error::Error;
use hextacy::{
    adapters::email::{RecipientInfo, SimpleTemplateMailer},
    contract, Constructor,
};
use std::{fmt::Display, sync::Arc};
use tracing::debug;

#[derive(Debug, Constructor)]
pub struct Email {
    pub driver: Arc<SimpleTemplateMailer>,

    /// Used as a base URL for the email redirect urls.
    #[env("DOMAIN")]
    domain: String,
}

pub enum EmailTemplate {
    AccountFrozen,
    ChangePassword,
    ForgotPassword,
    RegistrationToken,
    ResetPassword,
}

// The strings have to match the template names.
impl Display for EmailTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailTemplate::AccountFrozen => write!(f, "account_frozen"),
            EmailTemplate::ChangePassword => write!(f, "change_password"),
            EmailTemplate::ForgotPassword => write!(f, "forgot_password"),
            EmailTemplate::RegistrationToken => write!(f, "registration_token"),
            EmailTemplate::ResetPassword => write!(f, "reset_password"),
        }
    }
}

#[contract]
impl Email {
    fn send_registration_token(
        &self,
        token: &str,
        username: &str,
        email: &str,
    ) -> Result<(), Error> {
        debug!("Sending registration token email to {email}");
        let domain = &self.domain;
        let uri = format!("{domain}/auth/verify-registration-token?token={token}");

        self.driver
            .send(
                EmailTemplate::RegistrationToken,
                RecipientInfo::new(username.to_string(), email.to_string()),
                Some(&[("username", username), ("registration_uri", &uri)]),
                "Finish registration",
            )
            .map_err(Error::new)
    }

    fn send_reset_password(&self, username: &str, email: &str, temp_pw: &str) -> Result<(), Error> {
        debug!("Sending reset password email to {email}");

        self.driver
            .send(
                EmailTemplate::ResetPassword,
                RecipientInfo::new(username.to_string(), email.to_string()),
                Some(&[("temp_password", temp_pw)]),
                "Reset password",
            )
            .map_err(Error::new)
    }

    fn alert_password_change(&self, username: &str, email: &str, token: &str) -> Result<(), Error> {
        debug!("Sending change password email alert to {email}");
        let domain = &self.domain;
        let uri = format!("{domain}/auth/reset-password?token={token}");

        self.driver
            .send(
                EmailTemplate::ChangePassword,
                RecipientInfo::new(username.to_string(), email.to_string()),
                Some(&[("username", username), ("reset_password_uri", &uri)]),
                "Password change alert",
            )
            .map_err(Error::new)
    }

    fn send_forgot_password(&self, username: &str, email: &str, token: &str) -> Result<(), Error> {
        debug!("Sending forgot password email to {email}");

        self.driver
            .send(
                EmailTemplate::ForgotPassword,
                RecipientInfo::new(username.to_string(), email.to_string()),
                Some(&[("forgot_pw_token", token), ("username", username)]),
                "Forgot your password?",
            )
            .map_err(Error::new)
    }

    fn send_freeze_account(&self, username: &str, email: &str, token: &str) -> Result<(), Error> {
        debug!("Sending change password email alert to {email}");
        let domain = &self.domain;
        let uri = format!("{domain}/auth/reset-password?token={token}");

        self.driver
            .send(
                EmailTemplate::AccountFrozen,
                RecipientInfo::new(username.to_string(), email.to_string()),
                Some(&[("username", username), ("reset_password_uri", &uri)]),
                "Account suspended",
            )
            .map_err(Error::new)
    }
}
