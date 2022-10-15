use data_encoding::{Encoding, BASE32, BASE64, BASE64URL};
use hmac::{Hmac, Mac};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use sha2::Sha256;
use std::{env, fmt::Write, fs, path::Path};

/// cargo run -p infrastructure --bin generate_secret <KEY> [<ENCODING> b32 | b64 | b64u]
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

    let encoding = if let Some(enc) = args.get(2) {
        enc.clone()
    } else {
        String::from("b64u")
    };

    println!("Writing {key} with encoding {encoding}");

    let session_secret = secret(match &encoding[..] {
        "b32" => BASE32,
        "b64" => BASE64,
        "b64u" => BASE64URL,
        _ => BASE64URL,
    });

    write!(env_file, "\n{key} = \"{session_secret}\"").unwrap();

    fs::write(Path::new("./.env"), env_file).unwrap();
}

fn secret(enc: Encoding) -> String {
    type HmacSha256 = Hmac<Sha256>;

    let mut buff = [0_u8; 160];

    let mut rng = StdRng::from_entropy();
    rng.fill_bytes(&mut buff);

    let mac = HmacSha256::new_from_slice(&buff).unwrap().finalize();

    enc.encode(&mac.into_bytes())
}
