use super::{
    OAuth, OAuthAccount, OAuthProvider, OAuthProviderError, ProviderKeys, ProviderParams,
    RefreshTokenBody, TokenResponse,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::info;

pub struct GithubOAuth;

impl ProviderParams for GithubOAuth {}

impl ProviderKeys for GithubOAuth {
    fn token_url_key(&self) -> &'static str {
        "GITHUB_TOKEN_URI"
    }

    fn client_id_key(&self) -> &'static str {
        "GITHUB_CLIENT_ID"
    }

    fn client_secret_key(&self) -> &'static str {
        "GITHUB_CLIENT_SECRET"
    }

    fn redirect_uri_key(&self) -> &'static str {
        "GITHUB_REDIRECT_URI"
    }
}

#[async_trait]
impl OAuth for GithubOAuth {
    type Account = GithubAccount;
    type CodeExchangeResponse = GithubTokenResponse;

    async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<Self::CodeExchangeResponse, OAuthProviderError> {
        let client = reqwest::Client::new();

        let url = "oauth2.googleapis.com/token";
        let client_id = self.client_id()?;
        let client_secret = self.client_secret()?;

        let res = client
            .post(url)
            .form(&RefreshTokenBody {
                client_id: &client_id,
                client_secret: &client_secret,
                refresh_token,
                grant_type: "refresh_token",
            })
            .send()
            .await?;

        info!("Refreshing {} access token", self.provider_id());

        res.json::<Self::CodeExchangeResponse>()
            .await
            .map_err(|e| e.into())
    }

    async fn revoke_token(&self, token: &str) -> Result<reqwest::Response, OAuthProviderError> {
        let client = reqwest::Client::new();

        let client_id = self.client_id()?;

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
        exchange_res: &Self::CodeExchangeResponse,
    ) -> Result<Self::Account, OAuthProviderError> {
        let client = reqwest::Client::new();

        let url = "https://api.github.com/user";

        info!("Getting {} account", self.provider_id());

        let res = client
            .get(url)
            .header("accept", "application/vnd.github+json")
            .header("user-agent", "ALX")
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

impl OAuthAccount for GithubAccount {
    fn id(&self) -> String {
        self.id.to_string()
    }

    fn username(&self) -> &str {
        &self.username
    }

    fn email(&self) -> Option<&str> {
        self.email.as_ref().map(|s| s.as_str())
    }

    fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|s| s.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubTokenResponse {
    access_token: String,
    scope: String,
    token_type: String,
}

impl TokenResponse for GithubTokenResponse {
    fn access_token(&self) -> &str {
        &self.access_token
    }

    fn scope(&self) -> &str {
        &self.scope
    }

    fn token_type(&self) -> &str {
        &self.token_type
    }
}
