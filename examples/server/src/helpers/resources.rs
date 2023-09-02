use crate::config::constants::FAVICON_PATH;
use lazy_static::lazy_static;
use std::fs;
use tracing::debug;

pub fn initialize() {
    use self::*;
    lazy_static::initialize(&FAVICON);
    lazy_static::initialize(&super::validation::EMAIL_REGEX);
}

lazy_static! {
    pub static ref FAVICON: Vec<u8> = {
        debug!("Loading favicon");
        fs::read(FAVICON_PATH).expect("Couldn't load favicon.ico")
    };
}
