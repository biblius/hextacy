use lazy_static::lazy_static;
use tracing::debug;

pub fn initialize() {
    use self::*;
    lazy_static::initialize(&resources::FAVICON);
    lazy_static::initialize(&super::validation::EMAIL_REGEX);
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
