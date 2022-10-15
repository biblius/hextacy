use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use thiserror::Error;

use crate::config;

pub fn send_email(
    from: Option<&str>,
    to_uname: &str,
    to_email: &str,
    subject: &str,
    body: String,
) -> Result<(), EmailError> {
    let mut params = config::env::get_multiple(&[
        "EMAIL_SENDER",
        "SMTP_HOST",
        "SMTP_PORT",
        "SMTP_USERNAME",
        "SMTP_PASSWORD",
    ]);
    let password = params.pop().unwrap();
    let username = params.pop().unwrap();
    let port = params
        .pop()
        .unwrap()
        .parse::<u16>()
        .expect("Invalid SMTP port");
    let host = params.pop().expect("SMTP host must be set");
    let sender = params.pop().unwrap();

    let from = from.map_or_else(
        || format!("rps_chat <{}>", sender),
        |s| format!("{} <{}>", s, sender),
    );
    let to = format!("{} <{}>", to_uname, to_email);

    let email = Message::builder()
        .from(from.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .body(body)?;

    let m = SmtpTransport::relay(&host)?
        .credentials(Credentials::new(username, password))
        .port(port)
        .build();

    m.send(&email)?;
    Ok(())
}

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("Lettre Error: {0}")]
    Lettre(#[from] lettre::error::Error),
    #[error("SMTP Error: {0}")]
    SmtpTransport(#[from] lettre::transport::smtp::Error),
}
