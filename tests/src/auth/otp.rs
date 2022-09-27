use auth::otp::*;
use tracing::info;

const TEST_KEY: &'static str = "2d2f76005aacab11d7825898c9c33f2f1f624557208a33126d07efb80da394b709bc6c9acfdededc9268b3a70e0d9283ea59c7877333422cbe9dded4d30c2935325a5ee41f850f9da579736b6955d8e421361aba915e91c3a5d21633877686c313b3e942d1ed76a535ca9aec86a6709a097c7f923518d0f126de75491b716afee5477a669b24fc73110918feed9000144e85ba94ffe16ac7797886f5453273e9";

pub fn test() {
    info!("\n========== TEST - HOTP GENERATION ==========\n");
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
