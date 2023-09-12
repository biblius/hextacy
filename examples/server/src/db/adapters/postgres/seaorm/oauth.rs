#![allow(unused_variables)]
use crate::db::dto::oauth::OAuthMetaData;
use crate::db::{models::oauth::OAuthMeta, repository::oauth::OAuthRepository, RepoAdapterError};
use crate::services::oauth::OAuthProvider;
use async_trait::async_trait;
use sea_orm::prelude::*;

#[derive(Debug, Clone)]
pub struct PgOAuthAdapter;

#[async_trait]
impl<C> OAuthRepository<C> for PgOAuthAdapter
where
    C: ConnectionTrait + Send,
{
    /// Create user OAuth metadata
    async fn create<'a>(
        conn: &mut C,
        data: OAuthMetaData<'a>,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }

    /// Get an entry by it's DB ID
    async fn get_by_id(conn: &mut C, id: &str) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }

    /// Get an entry based on the OAuth account ID
    async fn get_by_account_id(
        conn: &mut C,
        account_id: &str,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }

    /// Get all oauth entries by the given user ID
    async fn get_by_user_id(
        conn: &mut C,
        user_id: &str,
    ) -> Result<Vec<OAuthMeta>, RepoAdapterError> {
        todo!()
    }

    /// Get all oauth entries by the given user ID and provider
    async fn get_by_provider(
        conn: &mut C,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }

    /// Revoke an access token
    async fn revoke(conn: &mut C, access_token: &str) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }

    /// Revoke all access tokens based on user ID
    async fn revoke_all(conn: &mut C, user_id: &str) -> Result<Vec<OAuthMeta>, RepoAdapterError> {
        todo!()
    }

    /// Update a token's scopes, i.e. replace the found entry's tokens with the newly
    /// obtained ones. Matches against the user ID and the provider.
    async fn update<'a>(
        conn: &mut C,
        data: OAuthMetaData<'a>,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }
}
