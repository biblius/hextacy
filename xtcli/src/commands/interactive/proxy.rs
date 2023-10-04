use crate::commands::crypto::{PWOpts, SecretOpts};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommandProxy {
    Envex,
    Crypto,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SubcommandProxy {
    Crypto(CryptoProxy),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CryptoProxy {
    Secret,
    Rsa,
    Password,
}

#[derive(Debug, Clone)]
pub enum OptionsProxy {
    CrySecret(SecretOpts),
    CryPW(PWOpts),
}

#[derive(Debug, Clone, Copy)]
pub enum OptionField {
    Name,
    All,
    Length,
    Encoding,
}

impl std::fmt::Display for OptionField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionField::Name => write!(f, "Name"),
            OptionField::All => write!(f, "All"),
            OptionField::Length => write!(f, "Length"),
            OptionField::Encoding => write!(f, "Encoding"),
        }
    }
}

pub trait CommandInfo: Debug {
    fn title(&self) -> &'static str;

    fn description(&self) -> &'static str {
        "Hello world, this is a description, hope you like it so lajkujte sherujte subskrajbujte"
    }

    fn command_repr(&self) -> String {
        "xtc".to_string()
    }
}

impl CommandInfo for CommandProxy {
    fn title(&self) -> &'static str {
        use CommandProxy as Cmd;
        match self {
            Cmd::Envex => "Envex",
            Cmd::Crypto => "Crypto",
        }
    }

    fn description(&self) -> &'static str {
        use CommandProxy as Cmd;
        match self {
            Cmd::Envex => "Create a `.env.example` file from the `.env` file in the root",
            Cmd::Crypto => "Create secrets to be used by the server",
        }
    }

    fn command_repr(&self) -> String {
        use CommandProxy as Cmd;
        match self {
            Cmd::Envex => "xtc envex".to_string(),
            Cmd::Crypto => "xtc c".to_string(),
        }
    }
}

impl CommandInfo for SubcommandProxy {
    fn title(&self) -> &'static str {
        match self {
            SubcommandProxy::Crypto(sub) => match sub {
                CryptoProxy::Secret => "Secret",
                CryptoProxy::Rsa => "RSA Keypair",
                CryptoProxy::Password => "Password",
            },
        }
    }

    fn command_repr(&self) -> String {
        match self {
            SubcommandProxy::Crypto(sub) => match sub {
                CryptoProxy::Secret => "secret".to_string(),
                CryptoProxy::Rsa => "rsa".to_string(),
                CryptoProxy::Password => "pw".to_string(),
            },
        }
    }
}

impl CommandInfo for OptionsProxy {
    fn title(&self) -> &'static str {
        match self {
            OptionsProxy::CrySecret(_) => "Secret",
            OptionsProxy::CryPW(_) => "Password",
        }
    }

    fn command_repr(&self) -> String {
        match self {
            OptionsProxy::CrySecret(opts) => {
                let enc = opts
                    .encoding
                    .as_ref()
                    .map_or_else(String::new, |s| format!("-e {s}"));
                format!("{} -l {} {enc}", opts.name, opts.length)
            }
            OptionsProxy::CryPW(opts) => format!("{} -l {}", opts.name, opts.length),
        }
    }
}
