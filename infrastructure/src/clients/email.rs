use crate::config;
pub use lettre;
use lettre::{transport::smtp::authentication::Credentials, SmtpTransport};
use tracing::debug;

/// Build an email client from the environment.
pub fn build_client() -> SmtpTransport {
    let mut params =
        config::env::get_multiple(&["SMTP_HOST", "SMTP_PORT", "SMTP_USERNAME", "SMTP_PASSWORD"]);

    let password = params.pop().expect("SMTP_PASSWORD must be set");
    let username = params.pop().expect("SMTP_USERNAME must be set");
    let port = params
        .pop()
        .expect("SMTP_PORT must be set")
        .parse::<u16>()
        .expect("Invalid SMTP port");
    let host = params.pop().expect("SMTP host must be set");

    debug!("Building SMTP client");

    SmtpTransport::relay(&host)
        .expect("Could not establish SmtpTransport")
        .credentials(Credentials::new(username, password))
        .port(port)
        .build()
}
