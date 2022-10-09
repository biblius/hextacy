#[allow(dead_code)]
use super::super::rsa_key_pair::generate_rsa_key_pair;

#[allow(dead_code)]
fn main() {
    generate_rsa_key_pair().expect("Couldn't generate keypair");
}
