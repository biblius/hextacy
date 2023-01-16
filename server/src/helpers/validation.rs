use lazy_static::lazy_static;
use regex::Regex;
use tracing::trace;

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
    Regex::new(r"^(00|\+)([1-9]{1})(d{6,14})$").unwrap()
  };

  /// Alphanumeric regex, allows spaces.
  static ref AN_NON_EMPTY: Regex = {
    trace!("Loading AN_NON_EMPTY regex");
    Regex::new("^[a-zA-Z0-9 ]+$").unwrap()
  };
}
