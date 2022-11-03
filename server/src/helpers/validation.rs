use lazy_static::lazy_static;
use regex::Regex;
use tracing::trace;
use validator::ValidationError;

lazy_static! {
    /// Crazy email regex
  pub static ref EMAIL_REGEX: Regex = {
    trace!("Loading EMAIL regex");
    Regex::new(
        r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#
    ).unwrap()
  };

  pub static ref PHONE_REGEX: Regex = {
    trace!("Loading PHONE regex");
    Regex::new("^(00|+)([1-9]{1})(d{6,14})$").unwrap()
  };

  /// Alphanumeric regex, allows spaces.
  static ref AN_NON_EMPTY: Regex = {
    trace!("Loading AN_NON_EMPTY regex");
    Regex::new("^[a-zA-Z0-9 ]+$").unwrap()
  };
}

/// Trims the string and verifies it's not empty and is alphanumeric with spaces.
pub fn _non_empty_alnum(input: &str) -> Result<(), ValidationError> {
    let i = input.trim();
    if i.is_empty() {
        return Err(ValidationError::new("Can't be empty"));
    }
    if !AN_NON_EMPTY.is_match(i) {
        return Err(ValidationError::new("Must be alphanumeric"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[derive(Debug, Validate)]
    struct Data {
        #[validate(custom = "_non_empty_alnum")]
        s: &'static str,
    }

    #[test]
    fn _non_empty() {
        let data = Data { s: "    " };
        assert!(matches!(data.validate(), Err(_)));

        let data = Data { s: "\n" };
        assert!(matches!(data.validate(), Err(_)));

        let data = Data { s: "  ok   " };
        assert!(matches!(data.validate(), Ok(_)));

        let data = Data { s: "n0_t  ok   " };
        assert!(matches!(data.validate(), Err(_)));

        let data = Data { s: "5t1ll ok \n  " };
        assert!(matches!(data.validate(), Ok(_)));
    }

    #[test]
    fn email() {}

    #[test]
    fn phone() {}
}
