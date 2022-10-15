use super::user::User;
use super::{role::Role, schema::sessions};
use crate::error::Error;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use infrastructure::storage::postgres::PgPoolConnection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Queryable)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub username: String,
    pub user_role: Role,
    pub frozen: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub soft_expires_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

impl Session {
    pub fn create(user: &User, conn: &mut PgPoolConnection) -> Result<Self, Error> {
        use super::schema::sessions::dsl::*;

        let new = NewSession {
            user_id: &user.id,
            username: &user.username,
            user_role: &user.role,
        };

        diesel::insert_into(sessions)
            .values(new)
            .get_result::<Self>(conn)
            .map_err(|e| e.into())
    }

    pub fn get_by_id(session_id: &str, conn: &mut PgPoolConnection) -> Result<Self, Error> {
        use super::schema::sessions::dsl::*;

        sessions
            .filter(id.eq(session_id))
            .first::<Self>(conn)
            .map_err(|e| e.into())
    }

    pub fn to_json(&self) -> Result<String, Error> {
        serde_json::to_string(self).map_err(|e| e.into())
    }
}

#[derive(Debug, Serialize, Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSession<'a> {
    user_id: &'a str,
    username: &'a str,
    user_role: &'a Role,
}
