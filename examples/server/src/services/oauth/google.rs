use super::{
    OAuth, OAuthAccount, OAuthProvider, OAuthProviderError, OAuthTokenResponse, RefreshTokenBody,
};
use async_trait::async_trait;
use data_encoding::BASE64URL_NOPAD;
use hextacy::Constructor;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Constructor)]
pub struct GoogleOAuth {
    #[env("GOOGLE_TOKEN_URI")]
    token_uri: String,
    #[env("GOOGLE_CLIENT_ID")]
    client_id: String,
    #[env("GOOGLE_CLIENT_SECRET")]
    client_secret: String,
    #[env("GOOGLE_REDIRECT_URI")]
    redirect_uri: String,
}

#[async_trait]
impl OAuth for GoogleOAuth {
    async fn exchange_code(&self, code: &str) -> Result<OAuthTokenResponse, OAuthProviderError> {
        let client = reqwest::Client::new();

        let GoogleOAuth {
            token_uri,
            client_id,
            client_secret,
            redirect_uri,
        } = self;

        let res = client
            .post(token_uri)
            .header("accept", "application/json")
            .header("content-type", "application/x-www-form-urlencoded")
            .header("content-length", 0)
            .basic_auth(client_id, Some(client_secret))
            .query(&[
                ("code", code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", redirect_uri),
            ])
            .send()
            .await;
        match res {
            Ok(res) => {
                let content_type = res.headers().get("content-type");

                if content_type.is_none() {
                    return Err(OAuthProviderError::Response(res.text().await?));
                }

                if !content_type.unwrap().to_str()?.contains("application/json") {
                    return Err(OAuthProviderError::Response(res.text().await?));
                }

                if res.status().is_success() {
                    res.json::<OAuthTokenResponse>().await.map_err(|e| e.into())
                } else {
                    Err(OAuthProviderError::Response(
                        res.json::<serde_json::Value>().await?.to_string(),
                    ))
                }
            }
            Err(e) => {
                error!("Error occurred in token exchange {e}");
                Err(e.into())
            }
        }
    }

    async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<OAuthTokenResponse, OAuthProviderError> {
        let client = reqwest::Client::new();

        let url = "oauth2.googleapis.com/token";
        let client_id = &self.client_id;
        let client_secret = &self.client_secret;

        client
            .post(url)
            .form(&RefreshTokenBody {
                client_id,
                client_secret,
                refresh_token,
                grant_type: "refresh_token",
            })
            .send()
            .await?
            .json::<OAuthTokenResponse>()
            .await
            .map_err(OAuthProviderError::Reqwest)
    }

    async fn revoke_token(&self, token: &str) -> Result<reqwest::Response, OAuthProviderError> {
        let client = reqwest::Client::new();
        let url = "oauth2.googleapis.com/revoke";
        client
            .post(url)
            .query(&[("token", token)])
            .send()
            .await
            .map_err(|e| e.into())
    }

    async fn get_account(
        &self,
        exchange_res: &OAuthTokenResponse,
    ) -> Result<OAuthAccount, OAuthProviderError> {
        let client = reqwest::Client::new();

        match exchange_res.id_token {
            Some(ref token) => {
                let jwt_body = match token.split('.').nth(1) {
                    Some(body) => body,
                    None => return Err(OAuthProviderError::InvalidJwt),
                };
                let decoded = BASE64URL_NOPAD.decode(jwt_body.as_bytes())?;
                let acc = serde_json::from_slice::<GoogleOpenID>(&decoded)?.into();
                Ok(acc)
            }
            None => {
                let url = "www.googleapis.com/userinfo/v2/me";
                let res = client
                    .get(url)
                    .header("Accept", "application/json")
                    .bearer_auth(&exchange_res.access_token)
                    .send()
                    .await?;

                let content_type = res.headers().get("content-type");

                if content_type.is_none() {
                    return Err(OAuthProviderError::Response(res.text().await?));
                }

                if !content_type.unwrap().to_str()?.contains("application/json") {
                    return Err(OAuthProviderError::Response(res.text().await?));
                }

                if res.status().is_success() {
                    // TODO: Should be google account then map
                    res.json::<OAuthAccount>().await.map_err(|e| e.into())
                } else {
                    Err(OAuthProviderError::Response(
                        res.json::<serde_json::Value>().await?.to_string(),
                    ))
                }
            }
        }
    }

    fn provider_id(&self) -> OAuthProvider {
        OAuthProvider::Google
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleAccount {
    id: String,
    email: String,
    #[serde(rename(deserialize = "verified_email"))]
    email_verified: bool,
    name: String,
    given_name: Option<String>,
    family_name: Option<String>,
    picture: Option<String>,
    locale: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleOpenID {
    azp: String,
    aud: String,
    sub: String,
    email: String,
    email_verified: bool,
    at_hash: String,
    name: String,
    picture: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
    locale: Option<String>,
    iat: u64,
    exp: u64,
}

impl From<GoogleOpenID> for OAuthAccount {
    fn from(
        GoogleOpenID {
            sub, email, name, ..
        }: GoogleOpenID,
    ) -> Self {
        Self {
            id: sub,
            username: name,
            email: Some(email),
        }
    }
}

impl From<GoogleOpenID> for GoogleAccount {
    fn from(
        GoogleOpenID {
            sub,
            email,
            email_verified,
            name,
            picture,
            given_name,
            family_name,
            locale,
            ..
        }: GoogleOpenID,
    ) -> Self {
        Self {
            id: sub,
            email,
            email_verified,
            name,
            given_name,
            family_name,
            picture,
            locale,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    scope: String,
    token_type: String,
    id_token: Option<String>,
}
