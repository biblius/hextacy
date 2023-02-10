use crate::{
    commands::{
        crypto::{PWOpts, SecretOpts},
        generate::GenerateArgs,
        migration::{GenMigration, RedoMigration},
    },
    DEFAULT_MIDDLEWARE_PATH, DEFAULT_ROUTER_PATH,
};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CommandProxy {
    Generate,
    Analyze,
    Envex,
    Migration,
    Crypto,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SubcommandProxy {
    Generate(GenerateProxy),
    Migration(MigrationProxy),
    Crypto(CryptoProxy),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GenerateProxy {
    Route,
    Middleware,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MigrationProxy {
    Gen,
    Run,
    Rev,
    Redo,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CryptoProxy {
    Secret,
    Rsa,
    Password,
}

#[derive(Debug, Clone)]
pub enum OptionsProxy {
    GenerateR(GenerateArgs),
    GenerateMW(GenerateArgs),
    GenMig(GenMigration),
    RedoMig(RedoMigration),
    CrySecret(SecretOpts),
    CryPW(PWOpts),
}

#[derive(Debug, Clone, Copy)]
pub enum OptionField {
    Name,
    Contracts,
    Path,
    All,
    Length,
    Encoding,
}

impl std::fmt::Display for OptionField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OptionField::Name => write!(f, "Name"),
            OptionField::Contracts => write!(f, "Contracts"),
            OptionField::Path => write!(f, "Path"),
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
        "alx".to_string()
    }
}

impl CommandInfo for CommandProxy {
    fn title(&self) -> &'static str {
        use CommandProxy::*;
        match self {
            Generate => "Generate",
            Analyze => "Analyze",
            Envex => "Envex",
            Migration => "Migration",
            Crypto => "Crypto",
        }
    }

    fn description(&self) -> &'static str {
        use CommandProxy::*;
        match self {
            Generate => "Create server endpoints and middleware",
            Analyze => {
                "Analyze the server directory and create a JSON/YAML file documenting endpoints"
            }
            Envex => "Create a `.env.example` file from the `.env` file in the root",
            Migration => "Run, revert or create postgres migrations",
            Crypto => "Create secrets to be used by the server",
        }
    }

    fn command_repr(&self) -> String {
        use CommandProxy::*;
        match self {
            Generate => "alx g".to_string(),
            Analyze => "alx a".to_string(),
            Envex => "alx envex".to_string(),
            Migration => "alx m".to_string(),
            Crypto => "alx c".to_string(),
        }
    }
}

impl CommandInfo for SubcommandProxy {
    fn title(&self) -> &'static str {
        match self {
            SubcommandProxy::Generate(sub) => match sub {
                GenerateProxy::Route => "Route",
                GenerateProxy::Middleware => "Middleware",
            },
            SubcommandProxy::Migration(sub) => match sub {
                MigrationProxy::Gen => "Generate",
                MigrationProxy::Run => "Run",
                MigrationProxy::Rev => "Revert",
                MigrationProxy::Redo => "Redo",
            },
            SubcommandProxy::Crypto(sub) => match sub {
                CryptoProxy::Secret => "Secret",
                CryptoProxy::Rsa => "RSA Keypair",
                CryptoProxy::Password => "Password",
            },
        }
    }
    /*
    fn description(&self) -> &'static str {
        "Hello world, this is a description, hope you like it so lajkujte sherujte subskrajbujte"
    } */

    fn command_repr(&self) -> String {
        match self {
            SubcommandProxy::Generate(sub) => match sub {
                GenerateProxy::Route => "r".to_string(),
                GenerateProxy::Middleware => "mw".to_string(),
            },
            SubcommandProxy::Migration(sub) => match sub {
                MigrationProxy::Gen => "gen".to_string(),
                MigrationProxy::Run => "run".to_string(),
                MigrationProxy::Rev => "rev".to_string(),
                MigrationProxy::Redo => "redo".to_string(),
            },
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
            OptionsProxy::GenerateR(_) => "Generate",
            OptionsProxy::GenerateMW(_) => "Generate",
            OptionsProxy::GenMig(_) => "Generate",
            OptionsProxy::RedoMig(_) => "Redo",
            OptionsProxy::CrySecret(_) => "Secret",
            OptionsProxy::CryPW(_) => "Password",
        }
    }

    fn command_repr(&self) -> String {
        match self {
            OptionsProxy::GenerateR(opts) => {
                let contracts = opts
                    .contracts
                    .as_ref()
                    .map_or_else(String::new, |s| format!("-c {s}"));
                let name = format!("{}", &opts.name);
                let path = opts
                    .path
                    .as_ref()
                    .map_or_else(String::new, |s| format!("-p {s}"));
                format!(
                    "{name} {contracts} {path} | Full path: {DEFAULT_ROUTER_PATH}/{}",
                    path.replace("-p ", "")
                )
            }
            OptionsProxy::GenerateMW(opts) => {
                let contracts = opts
                    .contracts
                    .as_ref()
                    .map_or_else(String::new, |s| format!("-c {s}"));
                let name = format!("{}", &opts.name);
                let path = opts
                    .path
                    .as_ref()
                    .map_or_else(String::new, |s| format!("-p {s}"));
                format!(
                    "{name} {contracts} {path} | Full path: {DEFAULT_MIDDLEWARE_PATH}/{}",
                    path.replace("-p ", "")
                )
            }
            OptionsProxy::GenMig(opts) => {
                format!("{}", opts.name)
            }
            OptionsProxy::RedoMig(opts) => {
                if opts.all {
                    "-a".to_string()
                } else {
                    String::new()
                }
            }
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
