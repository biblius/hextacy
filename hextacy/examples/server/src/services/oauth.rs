pub mod github;
pub mod google;

use crate::env;
use async_trait::async_trait;
use diesel::{
    deserialize::{self, FromSql},
    pg::{Pg, PgValue},
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    AsExpression, FromSqlRow,
};
use reqwest::header::{InvalidHeaderName, InvalidHeaderValue, ToStrError};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{env::VarError, fmt::Display, io::Write};
use thiserror::Error;
use tracing::error;

#[async_trait]
pub trait OAuth: ProviderParams {
    type Account: OAuthAccount + DeserializeOwned + 'static;
    type CodeExchangeResponse: TokenResponse + DeserializeOwned + 'static;

    async fn exchange_code(
        &self,
        code: &str,
    ) -> Result<Self::CodeExchangeResponse, OAuthProviderError>;

    async fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<Self::CodeExchangeResponse, OAuthProviderError>;

    async fn revoke_token(&self, token: &str) -> Result<reqwest::Response, OAuthProviderError>;

    async fn get_account(
        &self,
        token_res: &Self::CodeExchangeResponse,
    ) -> Result<Self::Account, OAuthProviderError>;

    fn provider_id(&self) -> OAuthProvider;
}

pub trait ProviderParams: ProviderKeys {
    fn token_url(&self) -> Result<String, VarError> {
        env::get(self.token_url_key())
    }

    fn client_id(&self) -> Result<String, VarError> {
        env::get(self.client_id_key())
    }

    fn client_secret(&self) -> Result<String, VarError> {
        env::get(self.client_secret_key())
    }

    fn redirect_uri(&self) -> Result<String, VarError> {
        env::get(self.redirect_uri_key())
    }
}

pub trait ProviderKeys {
    fn token_url_key(&self) -> &'static str;

    fn client_id_key(&self) -> &'static str;

    fn client_secret_key(&self) -> &'static str;

    fn redirect_uri_key(&self) -> &'static str;
}

pub trait TokenResponse: Send + Sync {
    fn access_token(&self) -> &str;

    fn scope(&self) -> &str;

    fn token_type(&self) -> &str;

    fn refresh_token(&self) -> Option<&str> {
        None
    }

    fn expires_in(&self) -> Option<i64> {
        None
    }

    fn id_token(&self) -> Option<&str> {
        None
    }
}

pub trait OAuthAccount: Send + Sync {
    fn id(&self) -> String;

    fn username(&self) -> &str;

    fn email(&self) -> Option<&str> {
        None
    }

    fn name(&self) -> Option<&str> {
        None
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
    #[error("Reqwest Header")]
    ToStr(#[from] ToStrError),
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
