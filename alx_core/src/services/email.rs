pub use lettre;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{message::header::ContentType, Message, SmtpTransport, Transport};
use std::{fmt::Write, fs, path::Path};
use tracing::debug;

use super::ServiceError;

/// Build an email client from the environment.
pub fn build_client() -> SmtpTransport {
    let mut params =
        utils::env::get_multiple(&["SMTP_HOST", "SMTP_PORT", "SMTP_USERNAME", "SMTP_PASSWORD"]);

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

/// Load a template from an HTML file and replace all the keywords with the targets.
/// Keywords must be formatted as `{{keyword}}`.
pub fn from_template(dir: &str, template_name: &str, replacements: &[(&str, &str)]) -> String {
    let template = fs::read_to_string(Path::new(&format!("{}/{}.html", dir, template_name)))
        .expect("Couldn't load template");

    let mut email = String::new();

    for line in template.lines() {
        let mut buf = String::new();

        for (target, replace_with) in replacements {
            // Target {{vars}} with rusty hyper-dimensional vag
            let target = format!("{{{{{}}}}}", target);

            if !line.contains(&target) {
                continue;
            }

            if buf.is_empty() {
                buf = line.replace(&target, replace_with);
            } else {
                buf = buf.replace(&target, replace_with)
            }
        }
        if !buf.is_empty() {
            writeln!(email, "{}", buf).unwrap();
        } else {
            writeln!(email, "{}", line).unwrap();
        }
        buf.clear()
    }
    email
}

/// Send an email with the given params
pub fn send(
    from: Option<&str>,
    to_uname: &str,
    to_email: &str,
    subject: &str,
    body: String,
    client: &SmtpTransport,
) -> Result<(), ServiceError> {
    let sender = utils::env::get_or_default("EMAIL_SENDER", "crazycompanyxxl@gmail.com");

    let from = from.map_or_else(
        || format!("Alx <{}>", sender),
        |s| format!("{} <{}>", s, sender),
    );
    let to = format!("{} <{}>", to_uname, to_email);

    debug!("Sending to: {to}");

    let email = Message::builder()
        .from(from.parse().unwrap())
        .to(to.parse().unwrap())
        .header(ContentType::TEXT_HTML)
        .subject(subject)
        .body(body)?;

    client.send(&email)?;
    Ok(())
}
