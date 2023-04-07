#![allow(unused_variables)]
use crate::services::oauth::OAuthProvider;
use crate::{
    db::{adapters::AdapterError, models::oauth::OAuthMeta, repository::oauth::OAuthRepository},
    services::oauth::TokenResponse,
};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use sea_orm::{prelude::*, DatabaseTransaction};

#[derive(Debug, Clone)]
pub struct PgOAuthAdapter;

#[async_trait]
impl OAuthRepository<DatabaseConnection> for PgOAuthAdapter {
    /// Create user OAuth metadata
    async fn create<T>(
        conn: &mut DatabaseConnection,
        user_id: &str,
        account_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, AdapterError>
    where
        T: TokenResponse + Send + Sync,
    {
        todo!()
    }

    /// Get an entry by it's DB ID
    async fn get_by_id(conn: &mut DatabaseConnection, id: &str) -> Result<OAuthMeta, AdapterError> {
        todo!()
    }

    /// Get an entry based on the OAuth account ID
    async fn get_by_account_id(
        conn: &mut DatabaseConnection,
        account_id: &str,
    ) -> Result<OAuthMeta, AdapterError> {
        todo!()
    }

    /// Get all oauth entries by the given user ID
    async fn get_by_user_id(
        conn: &mut DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<OAuthMeta>, AdapterError> {
        todo!()
    }

    /// Get all oauth entries by the given user ID and provider
    async fn get_by_provider(
        conn: &mut DatabaseConnection,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, AdapterError> {
        todo!()
    }

    /// Revoke an access token
    async fn revoke(
        conn: &mut DatabaseConnection,
        access_token: &str,
    ) -> Result<OAuthMeta, AdapterError> {
        todo!()
    }

    /// Revoke all access tokens based on user ID
    async fn revoke_all(
        conn: &mut DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<OAuthMeta>, AdapterError> {
        todo!()
    }

    /// Update a token's scopes, i.e. replace the found entry's tokens with the newly
    /// obtained ones. Matches against the user ID and the provider.
    async fn update<T>(
        conn: &mut DatabaseConnection,
        user_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, AdapterError>
    where
        T: TokenResponse,
    {
        todo!()
    }
}

#[async_trait]
impl OAuthRepository<DatabaseTransaction> for PgOAuthAdapter {
    /// Create user OAuth metadata
    async fn create<T>(
        conn: &mut DatabaseTransaction,
        user_id: &str,
        account_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, AdapterError>
    where
        T: TokenResponse + Send + Sync,
    {
        todo!()
    }

    /// Get an entry by it's DB ID
    async fn get_by_id(
        conn: &mut DatabaseTransaction,
        id: &str,
    ) -> Result<OAuthMeta, AdapterError> {
        todo!()
    }

    /// Get an entry based on the OAuth account ID
    async fn get_by_account_id(
        conn: &mut DatabaseTransaction,
        account_id: &str,
    ) -> Result<OAuthMeta, AdapterError> {
        todo!()
    }

    /// Get all oauth entries by the given user ID
    async fn get_by_user_id(
        conn: &mut DatabaseTransaction,
        user_id: &str,
    ) -> Result<Vec<OAuthMeta>, AdapterError> {
        todo!()
    }

    /// Get all oauth entries by the given user ID and provider
    async fn get_by_provider(
        conn: &mut DatabaseTransaction,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, AdapterError> {
        todo!()
    }

    /// Revoke an access token
    async fn revoke(
        conn: &mut DatabaseTransaction,
        access_token: &str,
    ) -> Result<OAuthMeta, AdapterError> {
        todo!()
    }

    /// Revoke all access tokens based on user ID
    async fn revoke_all(
        conn: &mut DatabaseTransaction,
        user_id: &str,
    ) -> Result<Vec<OAuthMeta>, AdapterError> {
        todo!()
    }

    /// Update a token's scopes, i.e. replace the found entry's tokens with the newly
    /// obtained ones. Matches against the user ID and the provider.
    async fn update<T>(
        conn: &mut DatabaseTransaction,
        user_id: &str,
        tokens: &T,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, AdapterError>
    where
        T: TokenResponse,
    {
        todo!()
    }
}
