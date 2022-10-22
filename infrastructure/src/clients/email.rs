use super::ClientError;
use crate::config;
pub use lettre;
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport, Transport,
};
use std::{fmt::Write, fs, path::Path};

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

    SmtpTransport::relay(&host)
        .expect("Could not establish SmtpTransport")
        .credentials(Credentials::new(username, password))
        .port(port)
        .build()
}

/// Load a template from an HTML file and replace all the keywords with the targets.
/// Keywords must be formatted as `{{keyword}}`.
pub fn from_template(template_name: &str, replacements: &[(&str, &str)]) -> String {
    let template = fs::read_to_string(Path::new(&format!("emails/{}.html", template_name)))
        .expect("Couldn't load template");

    let mut email = String::new();
    'first: for line in template.lines() {
        for (search, target) in replacements {
            let search = format!("{{{{{}}}}}", search);
            if line.contains(&search) {
                writeln!(email, "{}", line.replace(&search, target)).unwrap();
                continue 'first;
            }
        }
        writeln!(email, "{}", line).unwrap();
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
) -> Result<(), ClientError> {
    let sender = config::env::get_or_default("EMAIL_SENDER", "crazycompanyxxl@gmail.com");

    let from = from.map_or_else(
        || format!("rps_chat <{}>", sender),
        |s| format!("{} <{}>", s, sender),
    );
    let to = format!("{} <{}>", to_uname, to_email);

    let email = Message::builder()
        .from(from.parse().unwrap())
        .to(to.parse().unwrap())
        .header(ContentType::TEXT_HTML)
        .subject(subject)
        .body(body)?;

    client.send(&email)?;
    Ok(())
}
