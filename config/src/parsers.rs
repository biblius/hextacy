use crate::ConfigError;

pub fn parse_dot_env(buff: &str) -> Result<Vec<(String, String)>, ConfigError> {
    let err_msg = |index| format!("Invalid line in .env file at l:{}", index + 1);
    let mut vars: Vec<(String, String)> = vec![];
    for (i, mut line) in buff.lines().enumerate() {
        // Trim the lines in case of whitespace
        line = line.trim();

        println!("Reading: {}", line);

        // Ignore comments and empty lines
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        if let Some(idx) = line.find('=') {
            let (mut key, val) = line.split_at(idx);

            // Remove quotes and the remaining equals sign
            let val = &val.replace("\"", "")[1..];

            // Trim
            key = key.trim();
            let val = val.trim();

            println!("Key: {}", key);
            println!("Val: {}", val);

            if key.is_empty() || val.is_empty() {
                return Err(ConfigError::InvalidDotEnvLine(err_msg(i)));
            }

            vars.push((key.to_string(), val.to_string()))
        } else {
            return Err(ConfigError::InvalidDotEnvLine(err_msg(i)));
        }
    }
    Ok(vars)
}
