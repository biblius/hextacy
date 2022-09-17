mod parsers;

use parsers::parse_dot_env;
use std::{env, fmt::Display, fs::File, io::Read, path::Path};

/// Reads a file and sets all of its declared variables in the shell environment
pub fn load_from_file(path: &str, format: ConfigFormat) -> Result<(), ConfigError> {
    let mut file = String::new();

    File::open(Path::new(path))?.read_to_string(&mut file)?;

    let variables = match format {
        ConfigFormat::DotEnv => match parse_dot_env(&file) {
            Ok(vars) => vars,
            Err(e) => return Err(e),
        },
        ConfigFormat::Toml => todo!(),
        ConfigFormat::Yaml => todo!(),
    };

    for (key, value) in variables {
        set(&key, &value);
    }

    Ok(())
}

/// Retrieves a vec of values for the given keys set in the env, in the order
/// of the input vec.
pub fn get_from_env(keys: Vec<&str>) -> Vec<Option<String>> {
    let mut results = vec![];
    for key in keys {
        match env::var(key) {
            Ok(value) => {
                results.push(Some(value));
            }
            Err(_) => results.push(None),
        };
    }
    results
}

/// Sets an environment variable for the given key and value
pub fn set(key: &str, value: &str) {
    env::set_var(key, value)
}

pub enum ConfigFormat {
    DotEnv,
    Toml,
    Yaml,
}

#[derive(Debug)]
pub enum ConfigError {
    IOError(std::io::Error),
    InvalidDotEnvLine(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::IOError(e) => writeln!(f, "IOError: {e}"),
            ConfigError::InvalidDotEnvLine(msg) => {
                writeln!(f, "Error while parsing .env file: {msg}")
            }
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::IOError(e)
    }
}
