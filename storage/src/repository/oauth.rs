use crate::{adapters::AdapterError, models::oauth::OAuthMeta};
use alx_clients::oauth::{OAuthProvider, TokenResponse};

pub trait OAuthRepository<C> {
    /// Create user OAuth metadata
    fn create<T>(
        conn: &mut C,
        user_id: &str,
        account_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, AdapterError>
    where
        T: TokenResponse + Send + Sync;

    /// Get an entry by it's DB ID
    fn get_by_id(conn: &mut C, id: &str) -> Result<OAuthMeta, AdapterError>;

    /// Get an entry based on the OAuth account ID
    fn get_by_account_id(conn: &mut C, account_id: &str) -> Result<OAuthMeta, AdapterError>;

    /// Get all oauth entries by the given user ID
    fn get_by_user_id(conn: &mut C, user_id: &str) -> Result<Vec<OAuthMeta>, AdapterError>;

    /// Get all oauth entries by the given user ID and provider
    fn get_by_provider(
        conn: &mut C,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, AdapterError>;

    /// Revoke an access token
    fn revoke(conn: &mut C, access_token: &str) -> Result<OAuthMeta, AdapterError>;

    /// Revoke all access tokens based on user ID
    fn revoke_all(conn: &mut C, user_id: &str) -> Result<Vec<OAuthMeta>, AdapterError>;

    /// Update a token's scopes, i.e. replace the found entry's tokens with the newly
    /// obtained ones. Matches against the user ID and the provider.
    fn update<T>(
        conn: &mut C,
        user_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, AdapterError>
    where
        T: TokenResponse;
}
