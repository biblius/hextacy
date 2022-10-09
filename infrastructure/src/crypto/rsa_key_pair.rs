use rand;
use rsa::{
    pkcs1, pkcs1::EncodeRsaPublicKey, pkcs8, pkcs8::EncodePrivateKey, RsaPrivateKey, RsaPublicKey,
};
use std::fs;
use std::path::Path;
use tracing::info;
use tracing::log::warn;

#[derive(Debug)]
pub enum WriteError {
    FileSystemError(std::io::Error),
    Pkcs8(rsa::pkcs8::Error),
    Pkcs1(rsa::pkcs1::Error),
}

/// Generates an 2048 bit RSA key pair and stores in the root directory in a directory named key_pair.
pub fn generate_rsa_key_pair() -> Result<(), WriteError> {
    let mut rng = rand::thread_rng();
    let bits = 2048;

    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate private key");
    let pub_key = RsaPublicKey::from(&priv_key);

    info!("Attempting to remove 'key_pair' directory");

    match fs::remove_dir_all(Path::new("./key_pair")) {
        Ok(()) => info!("Deleting old key_pair directory"),
        Err(e) => warn!("{}.", e),
    }

    info!("Creating new directory 'key_pair'");

    if let Err(e) = fs::create_dir(Path::new("./key_pair")) {
        return Err(WriteError::FileSystemError(e));
    }

    if let Err(e) =
        priv_key.write_pkcs8_pem_file(Path::new("./key_pair/priv_key.pem"), pkcs8::LineEnding::LF)
    {
        return Err(WriteError::Pkcs8(e));
    }

    if let Err(e) =
        pub_key.write_pkcs1_pem_file(Path::new("./key_pair/pub_key.pem"), pkcs1::LineEnding::LF)
    {
        return Err(WriteError::Pkcs1(e));
    }

    Ok(())
}
