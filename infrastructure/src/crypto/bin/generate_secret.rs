//! Writes a random secret to the .env file located in the root.

use data_encoding::{Encoding, BASE32, BASE64, BASE64URL};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use std::{env, fmt::Write, fs, path::Path};

/// cargo run -p infrastructure --bin generate_secret <KEY> <LENGTH> [<ENCODING> b32 | b64 | b64u]
fn main() {
    let args = env::args().collect::<Vec<String>>();

    let mut env_file = fs::read_to_string(Path::new("./.env")).expect("No .env file in root!");

    let key = if let Some(key) = args.get(1) {
        key.clone()
    } else {
        let mut b = String::new();
        println!("Enter the key for the .env file:");
        std::io::stdin().read_line(&mut b).unwrap();
        b.trim().to_string()
    };

    let length = if let Some(len) = args.get(2) {
        len.parse::<usize>().expect("Invalid length")
    } else {
        256
    };

    let encoding = args.get(3).cloned();

    println!("Writing {key} with encoding {:?}", encoding);

    let session_secret = secret(
        match encoding {
            Some(enc) => match &enc[..] {
                "b32" => Some(BASE32),
                "b64" => Some(BASE64),
                "b64u" => Some(BASE64URL),
                _ => None,
            },
            None => None,
        },
        length,
    );

    write!(env_file, "\n{key} = \"{session_secret}\"").unwrap();

    fs::write(Path::new("./.env"), env_file).unwrap();
}

fn secret(enc: Option<Encoding>, len: usize) -> String {
    let mut buff = vec![0_u8; len];

    let mut rng = StdRng::from_entropy();
    rng.fill_bytes(&mut buff);

    if let Some(enc) = enc {
        enc.encode(&buff)
    } else {
        let mut b = String::new();
        for byte in buff {
            write!(b, "{:02x}", byte).unwrap();
        }
        b
    }
}
