use super::{
    OAuth, OAuthAccount, OAuthProvider, OAuthProviderError, ProviderKeys, ProviderParams,
    RefreshTokenBody, TokenResponse,
};
use async_trait::async_trait;
use data_encoding::BASE64URL_NOPAD;
use serde::{Deserialize, Serialize};
use tracing::error;

pub struct GoogleOAuth;

impl ProviderParams for GoogleOAuth {}

impl ProviderKeys for GoogleOAuth {
    fn token_url_key(&self) -> &'static str {
        "GOOGLE_TOKEN_URI"
    }

    fn client_id_key(&self) -> &'static str {
        "GOOGLE_CLIENT_ID"
    }

    fn client_secret_key(&self) -> &'static str {
        "GOOGLE_CLIENT_SECRET"
    }

    fn redirect_uri_key(&self) -> &'static str {
        "GOOGLE_REDIRECT_URI"
    }
}

#[async_trait]
impl OAuth for GoogleOAuth {
    type Account = GoogleAccount;
    type CodeExchangeResponse = GoogleTokenResponse;

    async fn exchange_code(
        &self,
        code: &str,
    ) -> Result<Self::CodeExchangeResponse, OAuthProviderError> {
        let client = reqwest::Client::new();

        let token_url = self.token_url()?;
        let client_id = self.client_id()?;
        let client_secret = self.client_secret()?;
        let redirect_uri = self.redirect_uri()?;

        let res = client
            .post(token_url)
            .header("accept", "application/json")
            .header("content-type", "application/x-www-form-urlencoded")
            .header("content-length", 0)
            .basic_auth(client_id, Some(client_secret))
            .query(&[
                ("code", code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &redirect_uri),
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
                    res.json::<Self::CodeExchangeResponse>()
                        .await
                        .map_err(|e| e.into())
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
    ) -> Result<Self::CodeExchangeResponse, OAuthProviderError> {
        let client = reqwest::Client::new();

        let url = "oauth2.googleapis.com/token";
        let client_id = self.client_id()?;
        let client_secret = self.client_secret()?;

        client
            .post(url)
            .form(&RefreshTokenBody {
                client_id: &client_id,
                client_secret: &client_secret,
                refresh_token,
                grant_type: "refresh_token",
            })
            .send()
            .await?
            .json::<Self::CodeExchangeResponse>()
            .await
            .map_err(|e| e.into())
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
        exchange_res: &Self::CodeExchangeResponse,
    ) -> Result<Self::Account, OAuthProviderError> {
        let client = reqwest::Client::new();

        match exchange_res.id_token() {
            Some(token) => {
                let jwt_body = match token.split('.').nth(1) {
                    Some(body) => body,
                    None => return Err(OAuthProviderError::InvalidJwt),
                };
                let decoded = BASE64URL_NOPAD.decode(jwt_body.as_bytes())?;
                let jwt = serde_json::from_slice::<GoogleOpenID>(&decoded)?.into();
                Ok(jwt)
            }
            None => {
                let url = "www.googleapis.com/userinfo/v2/me";
                let res = client
                    .get(url)
                    .header("Accept", "application/json")
                    .bearer_auth(exchange_res.access_token())
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
                    res.json::<Self::Account>().await.map_err(|e| e.into())
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

impl OAuthAccount for GoogleAccount {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn email(&self) -> Option<&str> {
        Some(&self.email)
    }

    fn username(&self) -> &str {
        &self.name
    }

    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }
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

impl TokenResponse for GoogleTokenResponse {
    fn access_token(&self) -> &str {
        &self.access_token
    }

    fn scope(&self) -> &str {
        &self.scope
    }

    fn token_type(&self) -> &str {
        &self.token_type
    }

    fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_ref().map(|s| s.as_str())
    }

    fn expires_in(&self) -> Option<i64> {
        Some(self.expires_in)
    }

    fn id_token(&self) -> Option<&str> {
        self.id_token.as_ref().map(|s| s.as_str())
    }
}
