use super::{role::Role, schema::users};
use crate::error::Error;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use infrastructure::storage::postgres::PgPoolConnection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Queryable)]
pub struct User {
    pub id: String,
    pub email: String,
    pub username: String,
    pub role: Role,
    #[serde(skip_serializing)]
    pub password: String,
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

#[derive(Debug, Deserialize, Serialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUserBasic<'a> {
    email: &'a str,
    username: &'a str,
    password: &'a str,
}

impl User {
    /// Creates a user entry with all the default properties
    pub fn create(
        user_email: &str,
        user_name: &str,
        user_pw: &str,
        conn: &mut PgPoolConnection,
    ) -> Result<Self, Error> {
        use super::schema::users::dsl::*;
        diesel::insert_into(users)
            .values(NewUserBasic {
                email: user_email,
                username: user_name,
                password: user_pw,
            })
            .get_result::<Self>(conn)
            .map_err(Error::new)
    }

    /// Fetches a user by their ID
    pub fn get_by_id(user_id: &str, conn: &mut PgPoolConnection) -> Result<Self, Error> {
        use super::schema::users::dsl::*;
        users
            .filter(id.eq(user_id))
            .first::<Self>(conn)
            .map_err(Error::new)
    }

    /// Fetches a user by their email
    pub fn get_by_email(user_email: &str, conn: &mut PgPoolConnection) -> Result<Self, Error> {
        use super::schema::users::dsl::*;
        users
            .filter(email.eq(user_email))
            .first::<Self>(conn)
            .map_err(Error::new)
    }

    /// Hashes the given password with bcrypt and sets the user's password field to the hash
    pub fn update_password(
        user_id: &str,
        pw_hash: &str,
        conn: &mut PgPoolConnection,
    ) -> Result<Vec<Self>, Error> {
        use super::schema::users::dsl::*;

        diesel::update(users.filter(id.eq(user_id)))
            .set(password.eq(pw_hash))
            .load::<Self>(conn)
            .map_err(Error::new)
    }

    /// Sets the user's frozen flag to true
    pub fn update_email_verified_at(
        user_id: &str,
        conn: &mut PgPoolConnection,
    ) -> Result<Vec<Self>, Error> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(email_verified_at.eq(chrono::Utc::now()))
            .load::<Self>(conn)
            .map_err(Error::new)
    }

    /// Updates the user's OTP secret to the given key
    pub fn update_otp_secret(
        user_id: &str,
        secret: &str,
        conn: &mut PgPoolConnection,
    ) -> Result<Vec<Self>, Error> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(otp_secret.eq(Some(secret)))
            .load::<Self>(conn)
            .map_err(Error::new)
    }

    /// Sets the user's frozen flag to true
    pub fn freeze(user_id: &str, conn: &mut PgPoolConnection) -> Result<Vec<Self>, Error> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(frozen.eq(true))
            .load::<Self>(conn)
            .map_err(Error::new)
    }

    /// Returns the total count of users and a vec of users constrained by the options as
    /// the first and second element respectively
    pub fn get_paginated(
        page: u16,
        per_page: u16,
        sort: SortOptions,
        conn: &mut PgPoolConnection,
    ) -> Result<(i64, Vec<Self>), Error> {
        use super::schema::users::dsl::*;

        let count = users.count().get_result::<i64>(conn)?;

        let mut query = users.into_boxed();

        match sort {
            SortOptions::UsernameAsc => query = query.order(username.asc()),
            SortOptions::UsernameDesc => query = query.order(username.desc()),
            SortOptions::EmailAsc => query = query.order(email.asc()),
            SortOptions::EmailDesc => query = query.order(email.desc()),
            SortOptions::CreatedAtAsc => query = query.order(created_at.asc()),
            SortOptions::CreatedAtDesc => query = query.order(created_at.desc()),
        };

        query = query.offset(((page - 1) * per_page) as i64);
        query = query.limit(per_page as i64);

        let result = query.load::<Self>(conn)?;

        Ok((count, result))
    }
}

#[derive(Debug, Deserialize)]
pub enum SortOptions {
    #[serde(rename = "username")]
    UsernameAsc,
    #[serde(rename = "-username")]
    UsernameDesc,
    #[serde(rename = "email")]
    EmailAsc,
    #[serde(rename = "-email")]
    EmailDesc,
    #[serde(rename = "createdAt")]
    CreatedAtAsc,
    #[serde(rename = "-createdAt")]
    CreatedAtDesc,
}
