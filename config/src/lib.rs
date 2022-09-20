use std::{
    env::{self, VarError},
    fmt::Display,
    path::Path,
};

/// Sets an environment variable for the given key and value
pub fn get(key: &str) -> Result<String, VarError> {
    env::var(key)
}

/// Sets an environment variable for the given key and value
pub fn set(key: &str, value: &str) {
    env::set_var(key, value)
}

/// Tries to load a variable from the shell env and if not found returns the provided default value
pub fn get_or_default(key: &str, default: &str) -> String {
    get(key).unwrap_or_else(|_| String::from(default))
}

/// Retrieves a vec of values for the given keys set in the env, in the order
/// of the input vec.
pub fn get_multiple(keys: Vec<&str>) -> Vec<String> {
    let mut results = vec![];
    for key in keys {
        match env::var(key) {
            Ok(value) => {
                results.push(value);
            }
            Err(e) => panic!("Error at key {}, {}", key, e),
        };
    }
    results
}

/// Retrieves a vec of values for the given keys set in the env, in the order
/// of the input vec, default to the given value if not found.
pub fn get_or_default_multiple(keys: Vec<(&str, &str)>) -> Vec<String> {
    let mut results = vec![];
    for (key, default) in keys {
        match env::var(key) {
            Ok(value) => {
                results.push(value);
            }
            Err(_) => results.push(default.to_string()),
        };
    }
    results
}

/// Reads a file and sets all of its declared variables in the shell environment
pub fn load_from_file(path: &str, format: ConfigFormat) -> Result<(), ConfigError> {
    match format {
        ConfigFormat::DotEnv => {
            dotenv::from_path(Path::new(path)).map_err(|e| ConfigError::DotEnv(e))
        }
        ConfigFormat::Toml => todo!(),
    }
}

/// Represents the type of file to parse variables from.
pub enum ConfigFormat {
    DotEnv,
    Toml,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    DotEnv(dotenv::Error),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => writeln!(f, "IOError: {e}"),
            ConfigError::DotEnv(e) => {
                writeln!(f, "Error while loading .env file: {e}")
            }
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<dotenv::Error> for ConfigError {
    fn from(e: dotenv::Error) -> Self {
        ConfigError::DotEnv(e)
    }
}
