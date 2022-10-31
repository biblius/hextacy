use lazy_static::lazy_static;
use tracing::debug;

pub fn initialize() {
    use self::*;
    lazy_static::initialize(&resources::FAVICON);
    lazy_static::initialize(&validation::EMAIL_REGEX);
}

pub mod resources {
    use super::*;
    use std::{fs, path::Path};

    lazy_static! {
        pub static ref FAVICON: Vec<u8> = {
            debug!("Loading favicon");
            fs::read(Path::new("resources/favicon.ico")).expect("Couldn't load favicon.ico")
        };
    }
}

pub mod validation {
    use super::*;
    use regex::Regex;

    lazy_static! {
        /// Crazy email regex
        pub static ref EMAIL_REGEX: Regex = {
          debug!("Loading email regex");
          Regex::new(
            r#"(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\.)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\.){3}(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\])"#
        ).unwrap()
      };
    }
}
