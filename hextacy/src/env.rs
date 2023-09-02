use std::{
    collections::HashMap,
    env::{self, VarError},
};

/// Gets an environment variable for the given key
pub fn get(key: &str) -> Result<String, VarError> {
    env::var(key)
}

/// Sets an environment variable to the given key and value
pub fn set(key: &str, value: &str) {
    env::set_var(key, value)
}

/// Tries to load a variable from the shell env and if not found returns the provided default value
pub fn get_or_default(key: &str, default: &str) -> String {
    get(key).unwrap_or_else(|_| String::from(default))
}

/// Retrieves a map of values for the given keys set in the env.
/// If the key is not found in the env, it will not be in the returned map.
pub fn get_multiple<'a>(keys: &[&'a str]) -> HashMap<&'a str, String> {
    let mut results = HashMap::new();
    for key in keys {
        let var = env::var(key);
        if let Ok(var) = var {
            results.insert(*key, var);
        }
    }
    results
}

/// The same as [get_multiple], but ensures there is a default value in the final map if the
/// key is not found in the end
pub fn get_or_default_multiple<'a>(keys: &'a [(&'a str, &str)]) -> HashMap<&'a str, String> {
    let mut results = HashMap::new();
    for (key, default) in keys {
        match env::var(key) {
            Ok(value) => {
                results.insert(*key, value);
            }
            Err(_) => {
                results.insert(key, default.to_string());
            }
        };
    }
    results
}

/// Reads a file and sets all of its declared variables in the shell environment
pub fn load_from_file(path: &str) -> Result<(), dotenv::Error> {
    dotenv::from_path(path)
}
