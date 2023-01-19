use super::{schema::sessions, PgAdapterError};
use crate::{
    clients::storage::postgres::Postgres,
    storage::{
        models::{session::Session, user::User},
        repository::{role::Role, session::SessionRepository, RepositoryError},
    },
};
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{ExpressionMethods, Insertable, QueryDsl, RunQueryDsl};
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize, Insertable)]
#[diesel(table_name = sessions)]
struct NewSession<'a> {
    user_id: &'a str,
    username: &'a str,
    user_role: &'a Role,
    csrf_token: &'a str,
    expires_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct PgSessionAdapter {
    pub client: Arc<Postgres>,
}

impl SessionRepository for PgSessionAdapter {
    /// Create a new user session. If the permanent flag is true, the session's `expires_at` field will be set to the maximum possible value
    fn create(&self, user: &User, csrf: &str, permanent: bool) -> Result<Session, RepositoryError> {
        use super::schema::sessions::dsl::*;

        let new = NewSession {
            user_id: &user.id,
            username: &user.username,
            user_role: &user.role,
            csrf_token: csrf,
            expires_at: if permanent {
                NaiveDateTime::MAX
            } else {
                (Utc::now() + Duration::minutes(30)).naive_utc()
            },
        };

        diesel::insert_into(sessions)
            .values(new)
            .get_result::<Session>(&mut self.client.connect()?)
            .map_err(|e| e.into())
    }

    /// Gets an unexpired session with its corresponding CSRF token
    fn get_valid_by_id(&self, session_id: &str, csrf: &str) -> Result<Session, RepositoryError> {
        use super::schema::sessions::dsl::*;
        sessions
            .filter(id.eq(session_id))
            .filter(csrf_token.eq(csrf))
            .filter(expires_at.gt(chrono::Utc::now()))
            .first::<Session>(&mut self.client.connect()?)
            .map_err(|e| e.into())
    }

    /// Updates the sessions `expires_at` field to 30 minutes from now
    fn refresh(&self, session_id: &str, csrf: &str) -> Result<Session, RepositoryError> {
        use super::schema::sessions::dsl::*;

        diesel::update(sessions)
            .filter(id.eq(session_id))
            .filter(csrf_token.eq(csrf))
            .set(expires_at.eq(Utc::now() + Duration::minutes(30)))
            .load::<Session>(&mut self.client.connect()?)?
            .pop()
            .ok_or_else(|| PgAdapterError::DoesNotExist("Session".to_string()).into())
    }

    /// Updates the sessions `expires_at` field to now
    fn expire(&self, session_id: &str) -> Result<Session, RepositoryError> {
        use super::schema::sessions::dsl::*;

        diesel::update(sessions)
            .filter(id.eq(session_id))
            .set(expires_at.eq(Utc::now()))
            .load::<Session>(&mut self.client.connect()?)?
            .pop()
            .ok_or_else(|| PgAdapterError::DoesNotExist("Session".to_string()).into())
    }

    /// Updates all user related sessions' `expires_at` field to now
    fn purge<'a>(
        &self,
        usr_id: &str,
        skip: Option<&'a str>,
    ) -> Result<Vec<Session>, RepositoryError> {
        use super::schema::sessions::dsl::*;

        let mut query = diesel::update(sessions)
            .filter(user_id.eq(usr_id))
            .filter(expires_at.ge(Utc::now()))
            .set(expires_at.eq(Utc::now()))
            .into_boxed();

        if let Some(skip) = skip {
            query = query.filter(id.ne(skip))
        }

        query
            .load::<Session>(&mut self.client.connect()?)
            .map_err(|e| e.into())
    }
}
