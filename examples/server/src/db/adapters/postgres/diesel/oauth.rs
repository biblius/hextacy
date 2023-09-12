use crate::{
    db::{
        dto::oauth::OAuthMetaData, models::oauth::OAuthMeta, repository::oauth::OAuthRepository,
        RepoAdapterError,
    },
    services::oauth::OAuthProvider,
};
use async_trait::async_trait;
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use hextacy::adapters::db::postgres::diesel::DieselConnection;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct PgOAuthAdapter;

#[async_trait]
impl OAuthRepository<DieselConnection> for PgOAuthAdapter {
    async fn create<'a>(
        conn: &mut DieselConnection,
        oauth: OAuthMetaData<'a>,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        use super::schema::oauth::dsl;

        diesel::insert_into(dsl::oauth)
            .values(oauth)
            .get_result::<OAuthMeta>(conn)
            .map_err(RepoAdapterError::from)
    }

    async fn get_by_id(
        conn: &mut DieselConnection,
        id: &str,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        use super::schema::oauth::dsl;

        dsl::oauth
            .filter(dsl::id.eq(id))
            .filter(dsl::revoked.eq(false))
            .first::<OAuthMeta>(conn)
            .map_err(RepoAdapterError::from)
    }

    async fn get_by_account_id(
        conn: &mut DieselConnection,
        account_id: &str,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        use super::schema::oauth::dsl;

        dsl::oauth
            .filter(dsl::account_id.eq(account_id))
            .filter(dsl::revoked.eq(false))
            .filter(dsl::expires_at.gt(Utc::now()))
            .first::<OAuthMeta>(conn)
            .map_err(RepoAdapterError::from)
    }

    async fn get_by_user_id(
        conn: &mut DieselConnection,
        user_id: &str,
    ) -> Result<Vec<OAuthMeta>, RepoAdapterError> {
        use super::schema::oauth::dsl;

        dsl::oauth
            .filter(dsl::user_id.eq(user_id))
            .filter(dsl::revoked.eq(false))
            .filter(dsl::expires_at.gt(Utc::now()))
            .load::<OAuthMeta>(conn)
            .map_err(RepoAdapterError::from)
    }

    async fn get_by_provider(
        conn: &mut DieselConnection,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        use super::schema::oauth::dsl;

        dsl::oauth
            .filter(dsl::provider.eq(provider))
            .filter(dsl::user_id.eq(user_id))
            .filter(dsl::revoked.eq(false))
            .filter(dsl::expires_at.gt(Utc::now()))
            .first::<OAuthMeta>(conn)
            .map_err(RepoAdapterError::from)
    }

    async fn revoke(
        conn: &mut DieselConnection,
        access_token: &str,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        use super::schema::oauth::dsl;

        diesel::update(dsl::oauth)
            .filter(dsl::access_token.eq(access_token))
            .set(dsl::revoked.eq(true))
            .load::<OAuthMeta>(conn)?
            .pop()
            .ok_or_else(|| RepoAdapterError::DoesNotExist)
    }

    async fn revoke_all(
        conn: &mut DieselConnection,
        user_id: &str,
    ) -> Result<Vec<OAuthMeta>, RepoAdapterError> {
        use super::schema::oauth::dsl;

        diesel::update(dsl::oauth)
            .filter(dsl::user_id.eq(user_id))
            .set(dsl::revoked.eq(true))
            .load::<OAuthMeta>(conn)
            .map_err(RepoAdapterError::from)
    }

    async fn update<'a>(
        conn: &mut DieselConnection,
        data: OAuthMetaData<'a>,
    ) -> Result<OAuthMeta, RepoAdapterError> {
        use super::schema::oauth::dsl;

        let OAuthMetaData {
            user_id,
            access_token,
            refresh_token,
            provider,
            scope,
            ..
        } = data;

        diesel::update(dsl::oauth)
            .filter(dsl::user_id.eq(user_id))
            .filter(dsl::provider.eq(provider))
            .set((
                dsl::access_token.eq(access_token),
                dsl::refresh_token.eq(refresh_token),
                dsl::scope.eq(scope),
            ))
            .load::<OAuthMeta>(conn)?
            .pop()
            .ok_or_else(|| RepoAdapterError::DoesNotExist)
    }
}
