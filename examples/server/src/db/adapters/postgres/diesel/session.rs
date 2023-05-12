use super::schema::sessions;
use crate::{
    db::{
        models::{
            role::Role,
            session::{AuthType, Session},
            user::User,
        },
        repository::session::SessionRepository,
        RepoAdapterError,
    },
    services::oauth::OAuthProvider,
};
use async_trait::async_trait;
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{ExpressionMethods, Insertable, QueryDsl, RunQueryDsl};
use hextacy::drivers::db::postgres::diesel::DieselConnection;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct PgSessionAdapter;

#[async_trait]
impl SessionRepository<DieselConnection> for PgSessionAdapter {
    /// Create a new user session. If `None` is given for `expires_after`, the session's `expires_at`
    /// field will be set to the maximum possible value, otherwise it will be set to expire in `expires_after` seconds.
    async fn create(
        conn: &mut DieselConnection,
        user: &User,
        csrf: &str,
        expires_after: Option<i64>,
        oauth_token: Option<&str>,
        provider: Option<OAuthProvider>,
    ) -> Result<Session, RepoAdapterError> {
        use super::schema::sessions::dsl;

        let new = NewSession {
            user_id: &user.id,
            username: &user.username,
            phone: user.phone.as_deref(),
            role: &user.role,
            email: &user.email,
            csrf,
            oauth_token,
            expires_at: expires_after.map_or_else(
                || NaiveDateTime::MAX,
                |after| (Utc::now() + Duration::seconds(after)).naive_utc(),
            ),
            auth_type: provider.map_or(AuthType::Native, AuthType::OAuth),
        };

        diesel::insert_into(dsl::sessions)
            .values(new)
            .get_result::<Session>(conn)
            .map_err(RepoAdapterError::from)
    }

    /// Gets an unexpired session with its corresponding CSRF token
    async fn get_valid_by_id(
        conn: &mut DieselConnection,
        id: &str,
        csrf: &str,
    ) -> Result<Session, RepoAdapterError> {
        use super::schema::sessions::dsl;
        dsl::sessions
            .filter(dsl::id.eq(id))
            .filter(dsl::csrf.eq(csrf))
            .filter(dsl::expires_at.gt(chrono::Utc::now()))
            .first::<Session>(conn)
            .map_err(RepoAdapterError::from)
    }

    /// Updates the sessions `expires_at` field to 30 minutes from now
    async fn refresh(
        conn: &mut DieselConnection,
        id: &str,
        csrf: &str,
    ) -> Result<Session, RepoAdapterError> {
        use super::schema::sessions::dsl;

        diesel::update(dsl::sessions)
            .filter(dsl::id.eq(id))
            .filter(dsl::csrf.eq(csrf))
            .set(dsl::expires_at.eq(Utc::now() + Duration::minutes(30)))
            .load::<Session>(conn)?
            .pop()
            .ok_or_else(|| RepoAdapterError::DoesNotExist)
    }

    /// Updates the sessions `expires_at` field to now
    async fn expire(conn: &mut DieselConnection, id: &str) -> Result<Session, RepoAdapterError> {
        use super::schema::sessions::dsl;

        diesel::update(dsl::sessions)
            .filter(dsl::id.eq(id))
            .set(dsl::expires_at.eq(Utc::now()))
            .load::<Session>(conn)?
            .pop()
            .ok_or_else(|| RepoAdapterError::DoesNotExist)
    }

    /// Updates all user related sessions' `expires_at` field to now
    async fn purge(
        conn: &mut DieselConnection,
        usr_id: &str,
        skip: Option<&str>,
    ) -> Result<Vec<Session>, RepoAdapterError> {
        use super::schema::sessions::dsl;

        let mut query = diesel::update(dsl::sessions)
            .filter(dsl::user_id.eq(usr_id))
            .filter(dsl::expires_at.ge(Utc::now()))
            .set(dsl::expires_at.eq(Utc::now()))
            .into_boxed();

        if let Some(skip) = skip {
            query = query.filter(dsl::id.ne(skip))
        }

        query.load::<Session>(conn).map_err(RepoAdapterError::from)
    }

    async fn update_access_tokens(
        conn: &mut DieselConnection,
        access_token: &str,
        user_id: &str,
        provider: OAuthProvider,
    ) -> Result<Vec<Session>, RepoAdapterError> {
        use super::schema::sessions::dsl;

        let ty = match provider {
            OAuthProvider::Google => AuthType::OAuth(OAuthProvider::Google),
            OAuthProvider::Github => AuthType::OAuth(OAuthProvider::Github),
        };

        diesel::update(dsl::sessions)
            .filter(dsl::user_id.eq(user_id))
            .filter(dsl::auth_type.eq(ty))
            .filter(dsl::expires_at.ge(Utc::now()))
            .set(dsl::oauth_token.eq(access_token))
            .load::<Session>(conn)
            .map_err(RepoAdapterError::from)
    }
}

#[derive(Debug, Serialize, Insertable)]
#[diesel(table_name = sessions)]
struct NewSession<'a> {
    user_id: &'a str,
    username: &'a str,
    email: &'a str,
    phone: Option<&'a str>,
    role: &'a Role,
    csrf: &'a str,
    oauth_token: Option<&'a str>,
    expires_at: NaiveDateTime,
    auth_type: AuthType,
}
