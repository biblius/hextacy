use clap::{Args, Subcommand};
use data_encoding::{Encoding, BASE32, BASE64, BASE64URL};
use rand::{rngs::StdRng, thread_rng, Rng, RngCore, SeedableRng};
use rsa::pkcs1::{self, EncodeRsaPublicKey};
use rsa::pkcs8::EncodePrivateKey;
use rsa::{pkcs8, RsaPrivateKey, RsaPublicKey};
use std::fmt::Write;
use std::fs;

pub const DEFAULT_SECRET_LENGTH: &str = "256";
pub const DEFAULT_PW_LENGTH: &str = "64";

#[derive(Debug, Args)]
/// Cryptography related actions.
pub struct Crypto {
    #[clap(subcommand)]
    pub action: CryptoSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum CryptoSubcommand {
    /// Generate a password and store it in './encryption/pw-<NAME>'
    PW(PWOpts),
    /// Create an RSA keypair and store it in './encryption/keypair'
    Rsa,
    /// Write a secret with the given key to the '.env' file
    Secret(SecretOpts),
}

#[derive(Debug, Args, Default, Clone)]
/// Password options
pub struct PWOpts {
    /// Password name
    pub name: String,
    /// Password length
    #[arg(long, short, default_value = DEFAULT_PW_LENGTH)]
    pub length: u8,
}

#[derive(Debug, Args, Default, Clone)]
/// Secret options
pub struct SecretOpts {
    /// The key in the .env
    pub name: String,
    /// The path to the .env file, defaults to the cwd
    #[arg(long = "path", short, default_value = ".")]
    pub path_to_env: String,
    /// Length of the secret
    #[arg(long, short, default_value = DEFAULT_SECRET_LENGTH)]
    pub length: u16,
    /// Secret encoding
    #[arg(long, short)]
    pub encoding: Option<String>,
}

pub fn write_pw(opts: PWOpts) {
    const PW_PATH: &str = "./encryption/passwords";
    const ALPHABET: [char; 60] = [
        'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R',
        'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j',
        'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '!', '@',
        '#', '$', '%', '^', '&', '*',
    ];

    let PWOpts { name, length } = opts;

    let mut file = match fs::read_to_string(PW_PATH) {
        Ok(f) => f,
        Err(_) => {
            fs::write(PW_PATH, "").expect("Could not create password file");
            fs::read_to_string(PW_PATH).expect("Could not read passwords file")
        }
    };

    let mut pw = String::new();
    let mut rng = thread_rng();
    for _ in 0..length {
        pw.push(ALPHABET[rng.gen_range(0..ALPHABET.len())]);
    }

    file.extend(format!("{name} = {pw}\n").chars());

    std::fs::write(PW_PATH, file).unwrap();
}

pub fn write_secret(opts: SecretOpts) {
    let SecretOpts {
        name,
        length,
        path_to_env,
        encoding,
    } = opts;

    let mut env_file = fs::read_to_string(format!("{}/.env", &path_to_env))
        .unwrap_or_else(|_| panic!("Could not find .env file at '{path_to_env}'"));

    let name = if name.trim().is_empty() {
        let mut buf = String::new();
        println!("Enter the key for the secret to store in the .env file:");
        std::io::stdin().read_line(&mut buf).unwrap();
        buf.trim().to_string()
    } else {
        name
    };

    let secret = secret(
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

    write!(env_file, "{name} = \"{secret}\"\n").unwrap();

    fs::write(format!("{path_to_env}/.env"), env_file).unwrap();
}

fn secret(enc: Option<Encoding>, len: u16) -> String {
    let mut buff = vec![0_u8; len.into()];

    let mut rng = StdRng::from_entropy();
    rng.fill_bytes(&mut buff);

    if let Some(enc) = enc {
        enc.encode(&buff)
    } else {
        let mut b = String::new();
        for byte in buff {
            write!(b, "{byte:02x}").unwrap();
        }
        b
    }
}

#[derive(Debug)]
pub enum WriteError {
    FileSystemError(std::io::Error),
    Pkcs8(rsa::pkcs8::Error),
    Pkcs1(rsa::pkcs1::Error),
}

/// Generates an 2048 bit RSA key pair.
pub fn generate_rsa_key_pair() -> Result<(), WriteError> {
    const KEY_PATH: &str = "./encryption/key_pair";
    let mut rng = StdRng::from_entropy();
    let bits = 2048;

    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate private key");
    let pub_key = RsaPublicKey::from(&priv_key);

    if fs::create_dir("./encryption/key_pair").is_err() {
        match fs::remove_dir_all(KEY_PATH) {
            Ok(()) => println!("Deleted old key_pair directory"),
            Err(_) => println!("No `keypair` directory found, creating"),
        }
    }

    if let Err(e) = fs::create_dir_all(KEY_PATH) {
        return Err(WriteError::FileSystemError(e));
    }

    if let Err(e) =
        priv_key.write_pkcs8_pem_file(format!("{KEY_PATH}/priv_key.pem"), pkcs8::LineEnding::LF)
    {
        return Err(WriteError::Pkcs8(e));
    }

    if let Err(e) =
        pub_key.write_pkcs1_pem_file(format!("{KEY_PATH}/pub_key.pem"), pkcs1::LineEnding::LF)
    {
        return Err(WriteError::Pkcs1(e));
    }

    Ok(())
}
