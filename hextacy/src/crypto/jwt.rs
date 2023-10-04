use super::CryptoError;
use jsonwebtoken::{encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Used for jwts. sub is the actual payload, iat and exp are unix timestamps representing the issued at and expiration times respectively.
/// As per https://www.rfc-editor.org/rfc/rfc7519#section-4.1.6
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub iat: u64,
    pub exp: u64,
}

impl Claims {
    pub fn new(sub: String, issuer: String, expires: u64) -> Self {
        Self {
            sub,
            iss: issuer,
            iat: jsonwebtoken::get_current_timestamp(),
            exp: expires,
        }
    }
}

/// Generates a JWT using the provided algorithm.
///
/// `priv_key` has to be a valid RSA private key.
pub fn generate(
    priv_key: &[u8],
    sub: String,
    issuer: String,
    expires_in: chrono::Duration,
    algo: Algorithm,
) -> Result<String, CryptoError> {
    let encoding_key = EncodingKey::from_rsa_pem(priv_key)?;

    let now = jsonwebtoken::get_current_timestamp();
    let exp_timestamp = now + expires_in.num_seconds() as u64;

    let claims = Claims::new(sub, issuer, exp_timestamp);
    let token = encode(&Header::new(algo), &claims, &encoding_key)?;

    Ok(token)
}

/// Parses the token issued by the generate_jwt function.
pub fn parse<T: Serialize + DeserializeOwned>(
    pub_key: &[u8],
    token: &str,
    algo: Algorithm,
) -> Result<T, CryptoError> {
    let decoding_key = DecodingKey::from_rsa_pem(pub_key)?;

    let token_data = jsonwebtoken::decode::<Claims>(token, &decoding_key, &Validation::new(algo))?;

    let result: T = serde_json::from_str(&token_data.claims.sub)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use jsonwebtoken::decode;
    use rand::{rngs::StdRng, SeedableRng};
    use rsa::{
        pkcs1::EncodeRsaPrivateKey,
        pkcs8::{self, EncodePublicKey},
        RsaPrivateKey, RsaPublicKey,
    };

    use crate::crypto::jwt::Claims;

    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct User {
        id: String,
        username: String,
    }
    #[test]
    fn encode_decode_jwt() {
        //Fetch the private key
        let mut rng = StdRng::from_entropy();
        let bits = 2048;

        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate private key");
        let pub_key = RsaPublicKey::from(&priv_key);

        //Transmogrify the key pair to the encoding and decoding keys as arrays of u8
        let encoding_key = jsonwebtoken::EncodingKey::from_rsa_pem(
            priv_key
                .to_pkcs1_pem(pkcs8::LineEnding::LF)
                .unwrap()
                .as_bytes(),
        )
        .expect("Couldn't parse encoding key");
        let decoding_key = jsonwebtoken::DecodingKey::from_rsa_pem(
            pub_key
                .to_public_key_pem(pkcs8::LineEnding::LF)
                .unwrap()
                .as_bytes(),
        )
        .expect("Couldn't parse decoding key");

        //Issued at
        let now = jsonwebtoken::get_current_timestamp();

        //Expires in 5 minutes
        let expires = now + 60 * 5;

        //Generate the claims
        let user = User {
            id: String::from("lol"),
            username: String::from("lawl"),
        };

        let claims = Claims::new(
            serde_json::to_string(&user).unwrap(),
            "biblius".to_string(),
            expires,
        );

        //Encode jwt
        let token = encode(
            &jsonwebtoken::Header::new(Algorithm::RS256),
            &claims,
            &encoding_key,
        )
        .expect("Couldn't encode token");
        //Set headers for decoding
        let validation = Validation::new(Algorithm::RS256);

        //Decode the token
        let decoded =
            decode::<Claims>(&token, &decoding_key, &validation).expect("Couldn't decode token");

        assert_eq!(claims, decoded.claims);
        assert_eq!(expires, decoded.claims.exp);
        assert_eq!(now, decoded.claims.iat);
        assert_eq!(Algorithm::RS256, decoded.header.alg);
    }
}
