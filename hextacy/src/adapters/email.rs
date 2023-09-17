use crate::Constructor;
use lettre::transport;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{message::header::ContentType, Message, SmtpTransport, Transport};
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::{fs, path::Path};
use thiserror::Error;
use tracing::debug;

/// A simple html template sender. Sends emails via SMTP.
///
/// To load templates, call [load_templates][SimpleTemplateMailer::load_templates] with the
/// directory containing your html templates. Each template should contain placeholders, i.e.
/// target keywords delimited by a set of delimiters (the default is "{{" and "}}". You can
/// configure the delimiter chars as well as the length.
pub struct SimpleTemplateMailer {
    smtp: SmtpTransport,
    sender_info: SenderInfo,
    templates: HashMap<String, String>,
    placeholders: HashMap<String, Vec<TemplatePlaceholder>>,
    target_delims: Option<(char, char)>,
    delim_len: usize,
}

impl Debug for SimpleTemplateMailer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SimpleTemplateMailer")
            .field("smtp", &"{ ... }")
            .field("sender_info", &self.sender_info)
            .field("templates", &self.templates)
            .field("placeholders", &self.placeholders)
            .field("target_delims", &self.target_delims)
            .finish()
    }
}

impl SimpleTemplateMailer {
    pub fn new(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        from: &str,
        sender: &str,
    ) -> Self {
        let smtp = SmtpTransport::relay(host)
            .expect("Could not establish SmtpTransport")
            .credentials(Credentials::new(username.to_string(), password.to_string()))
            .port(port)
            .build();

        debug!("Successfully initialised SMTP relay at {host}:{port}");

        Self {
            smtp,
            sender_info: SenderInfo {
                from: from.to_string(),
                sender: sender.to_string(),
            },
            templates: HashMap::new(),
            placeholders: HashMap::new(),
            target_delims: None,
            delim_len: 2,
        }
    }

    pub fn load_templates<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), TemplateMailerError> {
        let dir = fs::read_dir(dir)?;

        for entry in dir.filter_map(Result::ok) {
            if !entry.file_type()?.is_file() {
                continue;
            }

            let name = entry.file_name();
            let Some(name_str) = name.to_str() else {
                continue;
            };

            let Some((template, ext)) = name_str.split_once('.') else {
                continue;
            };

            if ext != "html" {
                continue;
            }

            let path = entry.path();
            let content = fs::read_to_string(path)?;

            // Find the placeholders
            let placeholders = find_template_placeholders(
                self.target_delims.unwrap_or(('{', '}')),
                self.delim_len,
                &content,
            )?;

            self.placeholders.insert(template.to_string(), placeholders);
            self.templates.insert(template.to_string(), content);
        }
        Ok(())
    }

    pub fn set_delimiters(&mut self, delims: (char, char), len: usize) {
        self.target_delims = Some(delims);
        self.delim_len = len;
    }

    /// Send an email with the given params
    pub fn send<T: Display>(
        &self,
        template: T,
        to: RecipientInfo,
        replacements: Option<&[(&str, &str)]>,
        subject: &str,
    ) -> Result<(), TemplateMailerError> {
        let from = self.sender_info.to_string();
        let to = to.to_string();
        let template = template.to_string();

        let Some(mut body) = self.templates.get(&template).cloned() else {
            return Err(TemplateMailerError::TemplateNotLoaded(template));
        };

        let email = Message::builder()
            .from(from.parse()?)
            .to(to.parse()?)
            .header(ContentType::TEXT_HTML);

        let Some(placeholders) = self.placeholders.get(&template) else {
            let email = email.subject(subject).body(body)?;
            self.smtp.send(&email)?;
            return Ok(());
        };

        let Some(replacements) = replacements else {
            let keys = placeholders.iter().map(|p| &p.key).collect::<Vec<_>>();
            return Err(TemplateMailerError::Placeholder(format!(
                "Placeholder args missing, expected {keys:?}"
            )));
        };

        replace_targets(&mut body, replacements, placeholders, self.delim_len)?;

        let email = email.subject(subject).body(body)?;
        self.smtp.send(&email)?;

        Ok(())
    }
}

fn replace_targets(
    body: &mut String,
    replacements: &[(&str, &str)],
    placeholders: &[TemplatePlaceholder],
    delim_len: usize,
) -> Result<(), TemplateMailerError> {
    let mut replaced: Vec<(usize, isize)> = vec![];
    for (target, replacement) in replacements {
        // Placeholders always get stored in the order they were found
        let Some(TemplatePlaceholder { start_i, end_i, .. }) =
            placeholders.iter().find(|ph| &ph.key == target)
        else {
            let keys = placeholders.iter().map(|p| &p.key).collect::<Vec<_>>();
            return Err(TemplateMailerError::Placeholder(format!(
                "Invalid placeholder arg, expected {keys:?}, found {target}"
            )));
        };

        let mut offset = 0;
        // We only care about offsets if the placeholder comes after the previous replacements
        for (r_start, off) in replaced.iter() {
            if start_i > r_start {
                offset += off
            }
        }

        let (start, end) = (
            (*start_i as isize + offset) as usize,
            (*end_i as isize + offset) as usize,
        );

        body.replace_range(start..end, replacement);
        replaced.push((
            *start_i,
            replacement.len() as isize - target.len() as isize - delim_len as isize * 2,
        ));
    }
    Ok(())
}

fn find_template_placeholders(
    delims: (char, char),
    delim_len: usize,
    template: &str,
) -> Result<Vec<TemplatePlaceholder>, TemplateMailerError> {
    let mut placeholders = vec![];
    let (start_delim, end_delim) = delims;
    let chars = template.char_indices();

    let mut open = false;

    let mut current = TemplatePlaceholder::default();

    let mut delims_found = 0;
    for (i, char) in chars {
        if open {
            if char == '\n' {
                return Err(TemplateMailerError::from_placeholder(
                    "Template placeholders must not contain newlines",
                    start_delim,
                    delim_len,
                    template,
                    current,
                ));
            }

            if char != end_delim {
                current.key.push(char);
                continue;
            }

            if char == end_delim {
                delims_found += 1;
            }

            if char == end_delim && delims_found == delim_len {
                open = false;
                delims_found = 0;
                current.end_i = i + 1; // range in replace_targets is not inclusive so we bump by 1

                placeholders.push(current);
                current = TemplatePlaceholder::default();
            }
        } else {
            if char != start_delim {
                delims_found = 0;
                current = TemplatePlaceholder::default();
                continue;
            }

            if char == start_delim {
                delims_found += 1;
            }

            if char == start_delim && delims_found == delim_len {
                open = true;
                delims_found = 0;
                current.start_i = i - (delim_len - 1);
            }
        }
    }

    if open {
        return Err(TemplateMailerError::from_placeholder(
            "Unterminated placeholder found",
            start_delim,
            delim_len,
            template,
            current,
        ));
    }

    Ok(placeholders)
}

#[derive(Debug, Error)]
pub enum TemplateMailerError {
    #[error("Sender or recipient: {0}")]
    Address(#[from] lettre::address::AddressError),

    #[error("SMTP: {0}")]
    Transport(#[from] transport::smtp::Error),

    #[error("Lettre: {0}")]
    Lettre(#[from] lettre::error::Error),

    #[error("Template not loaded: {0}")]
    TemplateNotLoaded(String),

    #[error("Placeholder: {0}")]
    Placeholder(String),

    #[error("Unterminated placeholder: {0}")]
    TemplatePlaceholder(String),

    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
}

impl TemplateMailerError {
    fn from_placeholder(
        message: &str,
        delim: char,
        len: usize,
        content: &str,
        holder: TemplatePlaceholder,
    ) -> TemplateMailerError {
        let start = holder.start_i.saturating_sub(30);
        let mut end = holder.start_i + 30;
        if end > content.len() {
            end = content.len()
        }

        let s = if start != 0 { "..." } else { "" };
        let f = if end < content.len() { "..." } else { "" };

        let from = &content[start..holder.start_i];
        let to = &content[holder.start_i + len..end];

        TemplateMailerError::TemplatePlaceholder(format!(
            "{message}: \"{s}{from} --> {} <-- {to}{f}\"",
            delim.to_string().repeat(len)
        ))
    }
}

#[derive(Debug, Default)]
struct TemplatePlaceholder {
    key: String,
    start_i: usize,
    end_i: usize,
}

#[derive(Debug, Constructor)]
pub struct SenderInfo {
    /// Represents the actual sender
    from: String,

    /// Represents the email from which
    sender: String,
}

#[derive(Debug, Constructor)]
/// Holds information about the recipient and the recipient's email.
pub struct RecipientInfo {
    recipient: String,
    recipient_org: String,
}

impl std::fmt::Display for SenderInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}>", self.from, self.sender)
    }
}

impl std::fmt::Display for RecipientInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}>", self.recipient, self.recipient_org)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_templates() {
        const TEMPLATE: &str =
            "<!doctype html><html><body>This is { tricky_test } a {{TEMPLATE}}</body></html>";
        let mut mail =
            SimpleTemplateMailer::new("127.0.0.1", 465, "foo", "secret foo", "foo", "bar");

        let _ = fs::create_dir("loads_templates_temp");
        fs::write("loads_templates_temp/test_mail.html", TEMPLATE).unwrap();
        mail.load_templates("loads_templates_temp").unwrap();

        let temp = mail.templates.get("test_mail").unwrap();
        let holder = &mail.placeholders.get("test_mail").unwrap()[0];

        assert_eq!(&temp[holder.start_i..holder.end_i], "{{TEMPLATE}}");
        assert_eq!(holder.key, "TEMPLATE");

        let _ = fs::remove_dir_all("loads_templates_temp");
    }

    #[test]
    fn errors_unterminated() {
        const TEMPLATE: &str =
            "<!doctype html><html><body>This is { tricky_test } a {{TEMPLATE</body></html>";
        let mut mail =
            SimpleTemplateMailer::new("127.0.0.1", 465, "foo", "secret foo", "foo", "bar");

        let _ = fs::create_dir("errors_unterminated_temp");
        fs::write("errors_unterminated_temp/test_mail.html", TEMPLATE).unwrap();
        let res = mail.load_templates("errors_unterminated_temp");
        let err = res.unwrap_err();
        assert!(err.to_string().contains("Unterminated placeholder found"));
        let _ = fs::remove_dir_all("errors_unterminated_temp");
    }

    #[test]
    fn errors_newline() {
        const TEMPLATE: &str =
            "<!doctype html><html><body>This is { tricky_test } a {{TEMPLATE\n}}</body></html>";
        let mut mail =
            SimpleTemplateMailer::new("127.0.0.1", 465, "foo", "secret foo", "foo", "bar");

        let _ = fs::create_dir("errors_double_temp");
        fs::write("errors_double_temp/test_mail.html", TEMPLATE).unwrap();
        let res = mail.load_templates("errors_double_temp");
        let err = res.unwrap_err();
        assert!(err
            .to_string()
            .contains("Template placeholders must not contain newlines"));
        let _ = fs::remove_dir_all("errors_double_temp");
    }

    #[test]
    fn replaces_targets() {
        let original =
        "<!doctype html><html><body>This is { tricky_test } a {{TEMPLATE}}. PLS replace {{this}} with something. {{yeah}} that should do it {{lol}}.</body></html>".to_string();
        let placeholders = find_template_placeholders(('{', '}'), 2, &original).unwrap();
        let replaced = "<!doctype html><html><body>This is { tricky_test } a hello. PLS replace the replacement with something. hell that should do it yeah.</body></html>";

        let mut body = original.clone();
        let replacements = &[
            ("TEMPLATE", "hello"),
            ("this", "the replacement"),
            ("yeah", "hell"),
            ("lol", "yeah"),
        ];
        replace_targets(&mut body, replacements, &placeholders, 2).unwrap();
        assert_eq!(body, replaced);

        let mut body = original.clone();
        let replacements = &[
            ("lol", "yeah"),
            ("TEMPLATE", "hello"),
            ("this", "the replacement"),
            ("yeah", "hell"),
        ];
        replace_targets(&mut body, replacements, &placeholders, 2).unwrap();
        assert_eq!(body, replaced);

        let mut body = original.clone();
        let replacements = &[
            ("lol", "yeah"),
            ("yeah", "hell"),
            ("this", "the replacement"),
            ("TEMPLATE", "hello"),
        ];
        replace_targets(&mut body, replacements, &placeholders, 2).unwrap();
        assert_eq!(body, replaced);
    }

    #[test]
    fn replaces_targets_custom_delim() {
        let original =
        "<!doctype html><html><body>This is { tricky_test } a <<<<TEMPLATE>>>>. PLS replace <<<<this>>>> with something. <<<<yeah>>>> that should do it <<<<lol>>>>.</body></html>".to_string();
        let placeholders = find_template_placeholders(('<', '>'), 4, &original).unwrap();
        let replaced = "<!doctype html><html><body>This is { tricky_test } a hello. PLS replace the replacement with something. hell that should do it yeah.</body></html>";

        let mut body = original.clone();
        let replacements = &[
            ("TEMPLATE", "hello"),
            ("this", "the replacement"),
            ("yeah", "hell"),
            ("lol", "yeah"),
        ];
        replace_targets(&mut body, replacements, &placeholders, 4).unwrap();
        assert_eq!(body, replaced);

        let mut body = original.clone();
        let replacements = &[
            ("lol", "yeah"),
            ("TEMPLATE", "hello"),
            ("this", "the replacement"),
            ("yeah", "hell"),
        ];
        replace_targets(&mut body, replacements, &placeholders, 4).unwrap();
        assert_eq!(body, replaced);

        let mut body = original.clone();
        let replacements = &[
            ("lol", "yeah"),
            ("yeah", "hell"),
            ("this", "the replacement"),
            ("TEMPLATE", "hello"),
        ];
        replace_targets(&mut body, replacements, &placeholders, 4).unwrap();
        assert_eq!(body, replaced);
    }

    #[test]
    fn replaces_targets_custom_delim_beginning_end() {
        let original = "<<<<TEMPLATE>>>> replaced <<<<this>>>>".to_string();
        let replaced = "hello replaced world".to_string();
        let placeholders = find_template_placeholders(('<', '>'), 4, &original).unwrap();

        let mut body = original.clone();
        let replacements = &[("TEMPLATE", "hello"), ("this", "world")];
        replace_targets(&mut body, replacements, &placeholders, 4).unwrap();
        assert_eq!(body, replaced);

        let mut body = original.clone();
        let replacements = &[("this", "world"), ("TEMPLATE", "hello")];
        replace_targets(&mut body, replacements, &placeholders, 4).unwrap();
        assert_eq!(body, replaced);
    }
}
