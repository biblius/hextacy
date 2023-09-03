use lettre::transport;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{message::header::ContentType, Message, SmtpTransport, Transport};
use std::{fmt::Write, fs, path::Path};
use tracing::debug;

// This implementation is concrete because I cba to write it via Drivers
pub struct Email {
    driver: SmtpTransport,
    pub domain: String,
}

impl std::fmt::Debug for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Email")
            .field("driver", &"SmtpTransport { .. }")
            .finish()
    }
}

impl Email {
    pub fn new(host: &str, port: u16, username: &str, password: &str, domain: &str) -> Self {
        debug!("Building SMTP driver");
        let driver = SmtpTransport::relay(host)
            .expect("Could not establish SmtpTransport")
            .credentials(Credentials::new(username.to_string(), password.to_string()))
            .port(port)
            .build();

        Self {
            driver,
            domain: domain.to_string(),
        }
    }

    /// Send an email with the given params
    pub fn send(
        &self,
        from: Option<&str>,
        to_uname: &str,
        to_email: &str,
        subject: &str,
        body: String,
    ) -> Result<(), transport::smtp::Error> {
        let sender = crate::env::get_or_default("EMAIL_SENDER", "crazycompanyxxl@gmail.com");

        let from = from.map_or_else(|| format!("Xtc <{sender}>"), |s| format!("{s} <{sender}>"));
        let to = format!("{to_uname} <{to_email}>");

        debug!("Sending to: {to}");

        let email = Message::builder()
            .from(from.parse().unwrap())
            .to(to.parse().unwrap())
            .header(ContentType::TEXT_HTML)
            .subject(subject)
            .body(body)
            .unwrap(); // TODO

        self.driver.send(&email)?;
        Ok(())
    }
}

/// Load a template from an HTML file and replace all the keywords with the targets.
/// Keywords must be formatted as `{{keyword}}`.
pub fn load_from_template(dir: &str, template_name: &str, replacements: &[(&str, &str)]) -> String {
    let template = fs::read_to_string(Path::new(&format!("{dir}/{template_name}.html")))
        .expect("Couldn't load template");

    let mut email = String::new();

    for line in template.lines() {
        let mut buf = String::new();

        for (target, replace_with) in replacements {
            // Target {{vars}} with rusty hyper-dimensional vag
            let target = format!("{{{{{target}}}}}");

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
            writeln!(email, "{buf}").unwrap();
        } else {
            writeln!(email, "{line}").unwrap();
        }
        buf.clear()
    }
    email
}
