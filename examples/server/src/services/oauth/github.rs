use super::{
    OAuth, OAuthAccount, OAuthProvider, OAuthProviderError, OAuthTokenResponse, RefreshTokenBody,
};
use async_trait::async_trait;
use hextacy::Constructor;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, info};

#[derive(Debug, Constructor)]
pub struct GithubOAuth {
    #[env("GITHUB_TOKEN_URI")]
    token_uri: String,
    #[env("GITHUB_CLIENT_ID")]
    client_id: String,
    #[env("GITHUB_CLIENT_SECRET")]
    client_secret: String,
    #[env("GITHUB_REDIRECT_URI")]
    redirect_uri: String,
}

/// Struct representing errors sent back by github
#[derive(Debug, Deserialize, Serialize, Error)]
pub struct GithubOAuthError {
    pub error: String,
    pub error_description: String,
    pub error_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubTokenResponse {
    access_token: String,
    scope: String,
    token_type: String,
}

impl From<GithubTokenResponse> for OAuthTokenResponse {
    fn from(
        GithubTokenResponse {
            access_token,
            scope,
            token_type,
        }: GithubTokenResponse,
    ) -> Self {
        Self {
            access_token,
            scope,
            token_type,
            refresh_token: None,
            expires_in: None,
            id_token: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GithubResponse {
    Success(GithubTokenResponse),
    Failure(GithubOAuthError),
}

impl std::fmt::Display for GithubOAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let GithubOAuthError {
            error,
            error_description,
            error_uri,
        } = self;
        write!(
            f,
            "error: {error}, description: {error_description}, uri: {error_uri}"
        )
    }
}

#[async_trait]
impl OAuth for GithubOAuth {
    async fn exchange_code(&self, code: &str) -> Result<OAuthTokenResponse, OAuthProviderError> {
        let client = reqwest::Client::new();
        let GithubOAuth {
            token_uri,
            client_id,
            client_secret,
            redirect_uri,
        } = self;

        dbg!(self);

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
                    let res = res.json::<GithubResponse>().await?;
                    match res {
                        GithubResponse::Success(res) => Ok(res.into()),
                        GithubResponse::Failure(err) => Err(err.into()),
                    }
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
        let GithubOAuth {
            client_id,
            client_secret,
            ..
        } = self;

        let res = client
            .post(url)
            .form(&RefreshTokenBody {
                client_id,
                client_secret,
                refresh_token,
                grant_type: "refresh_token",
            })
            .send()
            .await?;

        info!("Refreshing {} access token", self.provider_id());

        res.json::<OAuthTokenResponse>().await.map_err(|e| e.into())
    }

    async fn revoke_token(&self, token: &str) -> Result<reqwest::Response, OAuthProviderError> {
        let client = reqwest::Client::new();

        let client_id = &self.client_id;

        let url = format!("api.github.com/applications/{client_id}/grant");

        info!("Revoking {} access token", self.provider_id());

        client
            .post(url)
            .json(&("access_token", token))
            .send()
            .await
            .map_err(|e| e.into())
    }

    async fn get_account(
        &self,
        exchange_res: &OAuthTokenResponse,
    ) -> Result<OAuthAccount, OAuthProviderError> {
        let client = reqwest::Client::new();

        let url = "https://api.github.com/user";

        info!("Getting {} account", self.provider_id());

        let res = client
            .get(url)
            .header("accept", "application/vnd.github+json")
            .header("user-agent", "XTC")
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
            // TODO: Incorrect, deser to github acc
            res.json::<OAuthAccount>().await.map_err(|e| e.into())
        } else {
            Err(OAuthProviderError::Response(
                res.json::<serde_json::Value>().await?.to_string(),
            ))
        }
    }

    fn provider_id(&self) -> OAuthProvider {
        OAuthProvider::Github
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubAccount {
    id: u64,
    email: Option<String>,
    #[serde(rename(deserialize = "login"))]
    username: String,
    avatar_url: String,
    url: String,
    #[serde(rename(deserialize = "type"))]
    account_type: String,
    site_admin: bool,
    name: Option<String>,
    company: Option<String>,
    location: Option<String>,
}
