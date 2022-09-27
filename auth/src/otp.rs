//! To see the details of what is going on in this module see [RFC 4226](https://www.rfc-editor.org/rfc/rfc4226)

use hmac::{digest::CtOutput, Hmac, Mac};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use sha1::Sha1;
use std::fmt::Write;

pub const DIGITS: u32 = 6;
type HmacSha1 = Hmac<Sha1>;

/// Generates a one time password based on the given secret and counter value.
/// Returns the password and the counter incremented by 1.
pub fn generate_hotp(secret: &str, counter: u64) -> String {
    // Transform to bytes
    let key = decode_hex_key(secret);
    let data = &u64::to_be_bytes(counter);

    // Create an HMAC-SHA-1 digest with the given key and data
    let mut hmac = hmac_sha1_digest(&key, data).into_bytes();

    // Truncate to 4 bytes
    let trunc = dynamic_trunc(&mut hmac);

    // Convert to a number based on the 4 bytes
    let s_num = str_to_num(&trunc);

    // Mod it with the nuber of digits for the password
    let result = s_num % 10_usize.pow(DIGITS);

    // Pad with 0s if the number is shorter than 6 digits
    format!("{:06}", result)
}

/// Verifies the given password for the given counter and secret and if successfull increments the counter.
/// If verification fails the counter is left as is.
pub fn verify_hotp(password: &str, secret: &str, counter: u64) -> (bool, u64) {
    let pass = generate_hotp(secret, counter);
    if pass.eq(password) {
        let counter = (counter as u128 + 1) as u64;
        return (true, counter);
    }
    (false, counter)
}

/// Generates multiple hotp passwords in the range of `counter + lookahead` and compares them with the given
/// `password`. The counter wraps around on overflow.
///  If any passwords match the matching counter is returned, if all fail the counter is returned as is.
pub fn verify_hotp_lookahead(
    password: &str,
    secret: &str,
    counter: u64,
    lookahead: usize,
) -> (bool, u64) {
    for current in 0..lookahead {
        let count = (counter as u128 + current as u128) as u64;
        let pass = generate_hotp(secret, count);
        if pass.eq(password) {
            return (true, count);
        }
    }
    (false, counter)
}

/// Generates a secret key, i.e. a random 160 bit buffer encoded to a hex string
pub fn generate_secret() -> String {
    let buff = buffer_160();
    encode_hex_key(&buff)
}

/// Generates a cryptographically secure random 160 byte buffer, as recommended by the standard
fn buffer_160() -> [u8; 160] {
    let mut key = [0u8; 160];
    let mut rng = StdRng::from_entropy();
    rng.fill_bytes(&mut key);
    key
}

/// Generates a HMAC-SHA-1 digest with the given key and data.
fn hmac_sha1_digest(key: &[u8], data: &[u8]) -> CtOutput<HmacSha1> {
    let mut mac = HmacSha1::new_from_slice(key).expect("Unable to process MAC code");
    mac.update(data);
    mac.finalize()
}

/// The dynamic truncate function as described in [RFC 4226](https://www.rfc-editor.org/rfc/rfc4226).
/// Determines an offset based on the last 4 bits of the input. The offset is then used as the starting index
/// of a slice of the input that spans 4 bytes. Finally, that slice is returned with the first bit masked to 0
/// resulting in a sequence of 31 bits.
fn dynamic_trunc(hmac_result: &mut [u8]) -> [u8; 4] {
    // Grab the last 4 bits
    let offset_bits = hmac_result[19] & 0xf;

    // Convert them to a number as per the standard
    let offset = str_to_num(&[offset_bits]) as usize;

    // Take a slice from the original bytes based on the offset
    let result = &mut hmac_result[offset..=offset + 3];

    // Mask the 32nd bit
    result[0] = result[0] & 0x7f;

    result.try_into().expect("This should not have happended")
}

/// Transforms the given byte string to an integer high order byte first
fn str_to_num(bytes: &[u8]) -> usize {
    let mut buf = String::new();
    for byte in bytes {
        write!(buf, "{:04b}", byte).unwrap();
    }
    usize::from_str_radix(&buf, 2).expect("Couldn't parse binary to usize")
}

/// Encodes the given byte array to a hex string of length 320
fn encode_hex_key(buff: &[u8]) -> String {
    let mut s = String::with_capacity(buff.len() * 2);
    for byte in buff {
        write!(s, "{:02x}", byte).unwrap();
    }
    s
}

/// Parses a hex representation of a secret key to a u8 array. The key must be exactly 320 characters long.
fn decode_hex_key(s: &str) -> [u8; 160] {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect::<Vec<u8>>()
        .try_into()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    const TEST_KEY: &'static str = "2d2f76005aacab11d7825898c9c33f2f1f624557208a33126d07efb80da394b709bc6c9acfdededc9268b3a70e0d9283ea59c7877333422cbe9dded4d30c2935325a5ee41f850f9da579736b6955d8e421361aba915e91c3a5d21633877686c313b3e942d1ed76a535ca9aec86a6709a097c7f923518d0f126de75491b716afee5477a669b24fc73110918feed9000144e85ba94ffe16ac7797886f5453273e9";
    #[test]
    fn hex_string() {
        let buffer = buffer_160();
        let str = encode_hex_key(&buffer);
        let decoded = decode_hex_key(&str);
        assert_eq!(buffer, decoded)
    }
    #[test]
    fn hmac_length_and_str_to_num_() {
        // Hmac length
        let hmac = super::hmac_sha1_digest(b"super secret key", b"1");
        assert!(hmac.clone().into_bytes().len() == 20);

        // Str to num
        let six = [0, 0, 6];
        assert_eq!(str_to_num(&six), 6);

        let twenty_two = [0, 1, 6];
        assert_eq!(str_to_num(&twenty_two), 22);
    }

    #[test]
    fn generate_otp() {
        // Since this is deterministic it will always yield the same results
        let mut hmac = super::hmac_sha1_digest(b"super secret key", b"1").into_bytes();

        let trunc = dynamic_trunc(&mut hmac);
        assert_eq!([2, 165, 155, 87], trunc);

        let s_num = str_to_num(&trunc);
        assert_eq!(22_203_863, s_num);

        let d = s_num % 10_usize.pow(DIGITS);
        assert_eq!(203_863, d);
    }

    #[test]
    fn hotp_generation() {
        // Since this is deterministic it will always yield the same results
        let password = super::generate_hotp(TEST_KEY, 1);
        assert_eq!(password, "871791");

        let password = super::generate_hotp(TEST_KEY, 2);
        assert_eq!(password, "311373");
    }

    #[test]
    fn hotp_generation_verification() {
        // Since this is deterministic it will always yield the same results

        let counter = 1;
        let password = generate_hotp(TEST_KEY, counter);

        assert_eq!(password, "871791");

        let (result, counter) = verify_hotp("871791", TEST_KEY, counter);

        assert_eq!(counter, 2);
        assert_eq!(result, true);

        let password = generate_hotp(TEST_KEY, counter);

        assert_eq!(password, "311373");

        let (result, counter) = verify_hotp("311373", TEST_KEY, counter);

        assert_eq!(counter, 3);
        assert_eq!(result, true);

        let (result, counter) = verify_hotp("fail", TEST_KEY, counter);

        assert_eq!(result, false);
        assert_eq!(counter, 3);

        // Test with lookahead and overflow
        let password = generate_hotp(TEST_KEY, 3);
        let (result, counter) = verify_hotp_lookahead(&password, TEST_KEY, u64::MAX, 20);

        assert_eq!(result, true);
        assert_eq!(counter, 3);

        let password = generate_hotp(TEST_KEY, u64::MAX);

        let (result, counter) = verify_hotp_lookahead(&password, TEST_KEY, u64::MAX - 19, 20);

        assert_eq!(result, true);
        assert_eq!(counter, u64::MAX);

        let _ = generate_hotp(TEST_KEY, u64::MAX);

        let (result, counter) = verify_hotp("767342", TEST_KEY, u64::MAX);

        assert_eq!(result, true);
        assert_eq!(counter, 0);
    }
}
