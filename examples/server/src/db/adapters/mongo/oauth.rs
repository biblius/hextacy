#![allow(unused_variables)]

use crate::{
    db::{
        dto::oauth::OAuthMetaData, models::oauth::OAuthMeta, repository::oauth::OAuthRepository,
        RepoAdapterError,
    },
    services::oauth::OAuthProvider,
};
use async_trait::async_trait;
use mongodb::ClientSession;

#[derive(Debug, Clone)]
pub struct MgOauthAdapter;

#[async_trait]
impl OAuthRepository<ClientSession> for MgOauthAdapter {
    /// Create user OAuth metadata
    async fn create<'a>(
        conn: &mut ClientSession,
        data: OAuthMetaData<'a>,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }

    /// Get an entry by it's DB ID
    async fn get_by_id(conn: &mut ClientSession, id: &str) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }

    /// Get an entry based on the OAuth account ID
    async fn get_by_account_id(
        conn: &mut ClientSession,
        account_id: &str,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }

    /// Get all oauth entries by the given user ID
    async fn get_by_user_id(
        conn: &mut ClientSession,
        user_id: &str,
    ) -> Result<Vec<OAuthMeta>, RepoAdapterError> {
        todo!()
    }

    /// Get all oauth entries by the given user ID and provider
    async fn get_by_provider(
        conn: &mut ClientSession,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }

    /// Revoke an access token
    async fn revoke(
        conn: &mut ClientSession,
        access_token: &str,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }

    /// Revoke all access tokens based on user ID
    async fn revoke_all(
        conn: &mut ClientSession,
        user_id: &str,
    ) -> Result<Vec<OAuthMeta>, RepoAdapterError> {
        todo!()
    }

    /// Update a token's scopes, i.e. replace the found entry's tokens with the newly
    /// obtained ones. Matches against the user ID and the provider.
    async fn update<'a>(
        conn: &mut ClientSession,
        data: OAuthMetaData<'a>,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        todo!()
    }
}
