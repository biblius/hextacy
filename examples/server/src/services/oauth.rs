pub mod github;
pub mod google;

use crate::config::AppState;

use self::{
    github::{GithubOAuth, GithubOAuthError},
    google::GoogleOAuth,
};
use async_trait::async_trait;
use diesel::{
    deserialize::{self, FromSql},
    pg::{Pg, PgValue},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    AsExpression, FromSqlRow,
};
use hextacy::Constructor;
use reqwest::header::{InvalidHeaderName, InvalidHeaderValue, ToStrError};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, io::Write, sync::Arc};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Constructor)]
pub struct OAuthProviders {
    pub github: Arc<GithubOAuth>,
    pub google: Arc<GoogleOAuth>,
}

impl hextacy::web::Configure<AppState, actix_web::web::ServiceConfig> for OAuthProviders {
    fn configure(_: &AppState, cfg: &mut actix_web::web::ServiceConfig) {
        let github = GithubOAuth::new_from_env().unwrap();
        let google = GoogleOAuth::new_from_env().unwrap();
        let this = Self::new(Arc::new(github), Arc::new(google));
        cfg.app_data(actix_web::web::Data::new(this));
    }
}

#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub scope: String,
    pub token_type: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthAccount {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
}

#[async_trait]
pub trait OAuth {
    async fn exchange_code(&self, code: &str) -> Result<OAuthTokenResponse, OAuthProviderError>;

    async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<OAuthTokenResponse, OAuthProviderError>;

    async fn revoke_token(&self, token: &str) -> Result<reqwest::Response, OAuthProviderError>;

    async fn get_account(
        &self,
        token_res: &OAuthTokenResponse,
    ) -> Result<OAuthAccount, OAuthProviderError>;

    fn provider_id(&self) -> OAuthProvider;
}

#[async_trait]
impl<T> OAuth for Arc<T>
where
    T: OAuth + Send + Sync,
{
    async fn exchange_code(&self, code: &str) -> Result<OAuthTokenResponse, OAuthProviderError> {
        T::exchange_code(self, code).await
    }

    async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<OAuthTokenResponse, OAuthProviderError> {
        T::refresh_access_token(self, refresh_token).await
    }

    async fn revoke_token(&self, token: &str) -> Result<reqwest::Response, OAuthProviderError> {
        T::revoke_token(self, token).await
    }

    async fn get_account(
        &self,
        token_res: &OAuthTokenResponse,
    ) -> Result<OAuthAccount, OAuthProviderError> {
        T::get_account(self, token_res).await
    }

    fn provider_id(&self) -> OAuthProvider {
        T::provider_id(self)
    }
}

#[derive(Debug, Error)]
pub enum OAuthProviderError {
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Response Error: {0}")]
    Response(String),
    #[error("Env Error: {0}")]
    Env(#[from] std::env::VarError),
    #[error("Header Value Error: {0}")]
    HeaderValue(#[from] InvalidHeaderValue),
    #[error("Header Name Error: {0}")]
    HeaderName(#[from] InvalidHeaderName),
    #[error("Serde Error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Decoding Error: {0}")]
    Encoding(#[from] data_encoding::DecodeError),
    #[error("Malformed JWT")]
    InvalidJwt,
    #[error("Reqwest Header: {0}")]
    ToStr(#[from] ToStrError),
    #[error("Invalid Provider")]
    InvalidProvider,
    #[error("Github response: {0}")]
    GithubOAuth(#[from] GithubOAuthError),
}

#[derive(Debug, Clone, Copy, FromSqlRow, AsExpression, PartialEq, Eq, Serialize, Deserialize)]
#[diesel(sql_type = Text)]
pub enum OAuthProvider {
    Google,
    Github,
}

impl ToSql<Text, Pg> for OAuthProvider {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            OAuthProvider::Google => out.write_all(b"google")?,
            OAuthProvider::Github => out.write_all(b"github")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Pg> for OAuthProvider {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"google" => Ok(OAuthProvider::Google),
            b"github" => Ok(OAuthProvider::Github),
            _ => Err("Unrecognized provider".into()),
        }
    }
}

impl TryFrom<String> for OAuthProvider {
    type Error = OAuthProviderError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "google" => Ok(Self::Google),
            "github" => Ok(Self::Github),
            _ => Err(OAuthProviderError::InvalidProvider),
        }
    }
}

impl Display for OAuthProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthProvider::Google => write!(f, "Google"),
            OAuthProvider::Github => write!(f, "Github"),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct CodeExchangeBody<'a> {
    code: &'a str,
    client_id: &'a str,
    client_secret: &'a str,
    redirect_uri: &'a str,
    grant_type: &'a str,
}

#[derive(Debug, Serialize)]
pub(super) struct RefreshTokenBody<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    refresh_token: &'a str,
    grant_type: &'a str,
}
