use auth::otp::*;

const TEST_KEY: &'static str = "2d2f76005aacab11d7825898c9c33f2f1f624557208a33126d07efb80da394b709bc6c9acfdededc9268b3a70e0d9283ea59c7877333422cbe9dded4d30c2935325a5ee41f850f9da579736b6955d8e421361aba915e91c3a5d21633877686c313b3e942d1ed76a535ca9aec86a6709a097c7f923518d0f126de75491b716afee5477a669b24fc73110918feed9000144e85ba94ffe16ac7797886f5453273e9";

pub fn test() {
    // Since this is deterministic it will always yield the same results
    let HotpResult(password, counter) = generate_hotp(TEST_KEY, 1);
    assert_eq!(password, "871791");
    assert_eq!(counter, 2);

    let HotpResult(password, counter) = generate_hotp(TEST_KEY, counter);
    assert_eq!(password, "311373");
    assert_eq!(counter, 3);

    let secret = generate_hotp_secret();
    println!("{}", secret);
    println!("{}", secret.len());
}
