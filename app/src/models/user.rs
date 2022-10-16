use super::{role::Role, schema::users};
use crate::error::Error;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use infrastructure::{crypto, storage::postgres::PgPoolConnection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: String,
    pub role: Role,
    #[serde(skip_serializing)]
    pub password: Option<String>,
    #[serde(skip_serializing)]
    pub otp_secret: Option<String>,
    pub phone: Option<String>,
    pub google_id: Option<String>,
    pub github_id: Option<String>,
    pub frozen: bool,
    pub email_verified_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    /// Creates a user entry with all the default properties
    pub fn create(
        user_email: &str,
        user_name: &str,
        conn: &mut PgPoolConnection,
    ) -> Result<Self, Error> {
        use super::schema::users::dsl::*;
        diesel::insert_into(users)
            .values(NewUserBasic {
                email: user_email,
                username: user_name,
            })
            .get_result::<Self>(conn)
            .map_err(|e| e.into())
    }

    /// Fetches a user by their ID
    pub fn get_by_id(user_id: &str, conn: &mut PgPoolConnection) -> Result<Self, Error> {
        use super::schema::users::dsl::*;
        users
            .filter(id.eq(user_id))
            .first::<Self>(conn)
            .map_err(|e| e.into())
    }

    /// Fetches a user by their email
    pub fn get_by_email(user_email: &str, conn: &mut PgPoolConnection) -> Result<Self, Error> {
        use super::schema::users::dsl::*;
        users
            .filter(email.eq(user_email))
            .first::<Self>(conn)
            .map_err(|e| e.into())
    }

    /// Hashes the given password with bcrypt and sets the user's password field to the hash
    pub fn update_password(
        user_id: &str,
        pw: &str,
        conn: &mut PgPoolConnection,
    ) -> Result<Vec<Self>, Error> {
        use super::schema::users::dsl::*;

        let hashed = crypto::utils::bcrypt_hash(pw)?;

        diesel::update(users.filter(id.eq(user_id)))
            .set(password.eq(hashed))
            .load::<Self>(conn)
            .map_err(|e| e.into())
    }

    /// Sets the user's frozen flag to true.
    pub fn freeze(user_id: &str, conn: &mut PgPoolConnection) -> Result<Vec<Self>, Error> {
        use super::schema::users::dsl::*;

        diesel::update(users.filter(id.eq(user_id)))
            .set(frozen.eq(true))
            .load::<Self>(conn)
            .map_err(|e| e.into())
    }

    /// Sets the user's frozen flag to true.
    pub fn update_email_verified_at(
        user_id: &str,
        conn: &mut PgPoolConnection,
    ) -> Result<Vec<Self>, Error> {
        use super::schema::users::dsl::*;

        diesel::update(users.filter(id.eq(user_id)))
            .set(email_verified_at.eq(chrono::Utc::now()))
            .load::<Self>(conn)
            .map_err(|e| e.into())
    }
}

#[derive(Debug, Deserialize, Serialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUserBasic<'a> {
    email: &'a str,
    username: &'a str,
}
