use lazy_static::lazy_static;
use std::{fs, path::Path};
use tracing::debug;

pub fn initialize() {
    use self::*;
    lazy_static::initialize(&FAVICON);
    lazy_static::initialize(&super::validation::EMAIL_REGEX);
}

lazy_static! {
    pub static ref FAVICON: Vec<u8> = {
        debug!("Loading favicon");
        fs::read(Path::new("resources/favicon.ico")).expect("Couldn't load favicon.ico")
    };
}
