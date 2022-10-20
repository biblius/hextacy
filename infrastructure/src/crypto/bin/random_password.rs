//! Generates a random password and stores it in

use rand::{thread_rng, Rng};
use std::env;

const ALPHABET: [char; 60] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '!', '@', '#', '$', '%',
    '^', '&', '*',
];

/// Generates a random password using the alphabet above and the provided length.
/// Run the following from the ROOT DIRECTORY of the project.
/// cargo run -p infrastructure --bin random_password <length>
#[allow(dead_code)]
fn main() {
    let args = env::args().collect::<Vec<String>>();

    let length = &args[1].parse::<u8>().unwrap();

    let mut pw = String::new();

    let mut rng = thread_rng();

    for _ in 0..*length {
        pw.push(ALPHABET[rng.gen_range(0..60) as usize]);
    }

    std::fs::write(std::path::Path::new("./encryption/password"), pw).unwrap();
}
