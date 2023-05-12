use crate::{
    db::{models::oauth::OAuthMeta, RepoAdapterError},
    services::oauth::{OAuthProvider, TokenResponse},
};
use async_trait::async_trait;

#[async_trait]
pub trait OAuthRepository<C> {
    /// Create user OAuth metadata
    async fn create<T>(
        conn: &mut C,
        user_id: &str,
        account_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, RepoAdapterError>
    where
        T: TokenResponse + Send + Sync;

    /// Get an entry by it's DB ID
    async fn get_by_id(conn: &mut C, id: &str) -> Result<OAuthMeta, RepoAdapterError>;

    /// Get an entry based on the OAuth account ID
    async fn get_by_account_id(
        conn: &mut C,
        account_id: &str,
    ) -> Result<OAuthMeta, RepoAdapterError>;

    /// Get all oauth entries by the given user ID
    async fn get_by_user_id(
        conn: &mut C,
        user_id: &str,
    ) -> Result<Vec<OAuthMeta>, RepoAdapterError>;

    /// Get all oauth entries by the given user ID and provider
    async fn get_by_provider(
        conn: &mut C,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, RepoAdapterError>;

    /// Revoke an access token
    async fn revoke(conn: &mut C, access_token: &str) -> Result<OAuthMeta, RepoAdapterError>;

    /// Revoke all access tokens based on user ID
    async fn revoke_all(conn: &mut C, user_id: &str) -> Result<Vec<OAuthMeta>, RepoAdapterError>;

    /// Update a token's scopes, i.e. replace the found entry's tokens with the newly
    /// obtained ones. Matches against the user ID and the provider.
    async fn update<T>(
        conn: &mut C,
        user_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, RepoAdapterError>
    where
        T: TokenResponse;
}
