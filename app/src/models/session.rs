use super::user::User;
use super::{role::Role, schema::sessions};
use crate::error::Error;
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::prelude::*;
use infrastructure::storage::postgres::PgPoolConnection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub user_role: Role,
    #[serde(skip)]
    pub csrf_token: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

impl Session {
    /// Create a new user session
    pub fn create(user: &User, csrf: &str, conn: &mut PgPoolConnection) -> Result<Self, Error> {
        use super::schema::sessions::dsl::*;

        let new = NewSession {
            user_id: &user.id,
            username: &user.username,
            user_role: &user.role,
            csrf_token: csrf,
        };

        diesel::insert_into(sessions)
            .values(new)
            .get_result::<Self>(conn)
            .map_err(Error::new)
    }

    /// Gets an unexpired session with its corresponding CSRF token
    pub fn get_valid_by_id(
        session_id: &str,
        csrf: &str,
        conn: &mut PgPoolConnection,
    ) -> Result<Self, Error> {
        use super::schema::sessions::dsl::*;

        sessions
            .filter(id.eq(session_id))
            .filter(expires_at.gt(chrono::Utc::now()))
            .filter(csrf_token.eq(csrf))
            .first::<Self>(conn)
            .map_err(Error::new)
    }

    /// Updates the sessions `expires_at` field to 30 minutes from now
    pub fn refresh(session_id: &str, conn: &mut PgPoolConnection) -> Result<Vec<Self>, Error> {
        use super::schema::sessions::dsl::*;

        diesel::update(sessions)
            .filter(id.eq(session_id))
            .set(expires_at.eq(Utc::now() + Duration::minutes(30)))
            .load::<Self>(conn)
            .map_err(Error::new)
    }

    /// Updates the sessions `expires_at` field to now
    pub fn expire(session_id: &str, conn: &mut PgPoolConnection) -> Result<Vec<Self>, Error> {
        use super::schema::sessions::dsl::*;

        diesel::update(sessions)
            .filter(id.eq(session_id))
            .set(expires_at.eq(Utc::now()))
            .load::<Self>(conn)
            .map_err(Error::new)
    }

    /// Updates all user related sessions' `expires_at` field to now
    pub fn purge(usr_id: &str, conn: &mut PgPoolConnection) -> Result<Vec<Self>, Error> {
        use super::schema::sessions::dsl::*;

        diesel::update(sessions)
            .filter(user_id.eq(usr_id))
            .set(expires_at.eq(Utc::now()))
            .load::<Self>(conn)
            .map_err(Error::new)
    }
}

#[derive(Debug, Serialize, Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSession<'a> {
    user_id: &'a str,
    username: &'a str,
    user_role: &'a Role,
    csrf_token: &'a str,
}
