use rand::rngs::StdRng;
use rand::{self, SeedableRng};
use rsa::{
    pkcs1, pkcs1::EncodeRsaPublicKey, pkcs8, pkcs8::EncodePrivateKey, RsaPrivateKey, RsaPublicKey,
};
use std::fs;
use std::path::Path;

const KEY_PATH: &str = "./crypto/key_pair";

#[allow(dead_code)]
/// Generates an RSA key pair.
/// Run the following from the ROOT DIRECTORY of the project.
/// cargo run -p infrastructure --bin rsa_key_pair
fn main() {
    generate_rsa_key_pair().expect("Couldn't generate keypair");
}

#[derive(Debug)]
pub enum WriteError {
    FileSystemError(std::io::Error),
    Pkcs8(rsa::pkcs8::Error),
    Pkcs1(rsa::pkcs1::Error),
}

/// Generates an 2048 bit RSA key pair.
fn generate_rsa_key_pair() -> Result<(), WriteError> {
    let mut rng = StdRng::from_entropy();
    let bits = 2048;

    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate private key");
    let pub_key = RsaPublicKey::from(&priv_key);

    if fs::create_dir(Path::new("./crypto")).is_err() {
        println!("Directory `crypto` already exists, generating key pair");
        println!("Attempting to remove old key pair dir");

        match fs::remove_dir_all(Path::new(KEY_PATH)) {
            Ok(()) => println!("Deleted old key_pair directory"),

            Err(_) => println!("No `keypair` directory found"),
        }
    }

    println!("Generating keypair");

    if let Err(e) = fs::create_dir(Path::new(KEY_PATH)) {
        return Err(WriteError::FileSystemError(e));
    }

    if let Err(e) = priv_key.write_pkcs8_pem_file(
        Path::new(&format!("{}/priv_key.pem", KEY_PATH)),
        pkcs8::LineEnding::LF,
    ) {
        return Err(WriteError::Pkcs8(e));
    }

    if let Err(e) = pub_key.write_pkcs1_pem_file(
        Path::new(&format!("{}/pub_key.pem", KEY_PATH)),
        pkcs1::LineEnding::LF,
    ) {
        return Err(WriteError::Pkcs1(e));
    }

    Ok(())
}
