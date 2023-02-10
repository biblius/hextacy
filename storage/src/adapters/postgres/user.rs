use super::{schema::users, PgAdapterError};
use crate::{
    adapters::AdapterError,
    models::user::{SortOptions, User},
    repository::user::UserRepository,
};
use alx_clients::db::postgres::Postgres;
use diesel::{ExpressionMethods, Insertable, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Insertable)]
#[diesel(table_name = users)]
struct NewUser<'a> {
    email: &'a str,
    username: &'a str,
    password: &'a str,
}

#[derive(Debug, Clone)]
pub struct PgUserAdapter {
    pub client: Arc<Postgres>,
}

impl UserRepository for PgUserAdapter {
    fn create(
        &self,
        user_email: &str,
        user_name: &str,
        user_pw: &str,
    ) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;
        diesel::insert_into(users)
            .values(NewUser {
                email: user_email,
                username: user_name,
                password: user_pw,
            })
            .get_result::<User>(&mut self.client.connect()?)
            .map_err(|e| e.into())
    }

    /// Fetches a user by their ID
    fn get_by_id(&self, user_id: &str) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;
        users
            .filter(id.eq(user_id))
            .first::<User>(&mut self.client.connect()?)
            .map_err(|e| e.into())
    }

    /// Fetches a user by their email
    fn get_by_email(&self, user_email: &str) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;
        users
            .filter(email.eq(user_email))
            .first::<User>(&mut self.client.connect()?)
            .map_err(|e| e.into())
    }

    /// Hashes the given password with bcrypt and sets the user's password field to the hash
    fn update_password(&self, user_id: &str, pw_hash: &str) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(password.eq(pw_hash))
            .load::<User>(&mut self.client.connect()?)?
            .pop()
            .ok_or_else(|| PgAdapterError::DoesNotExist("User".to_string()).into())
    }

    /// Update the user's email verified at field to now
    fn update_email_verified_at(&self, user_id: &str) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(email_verified_at.eq(chrono::Utc::now()))
            .load::<User>(&mut self.client.connect()?)?
            .pop()
            .ok_or_else(|| PgAdapterError::DoesNotExist("User".to_string()).into())
    }

    /// Updates the user's OTP secret to the given key
    fn update_otp_secret(&self, user_id: &str, secret: &str) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(otp_secret.eq(Some(secret)))
            .load::<User>(&mut self.client.connect()?)?
            .pop()
            .ok_or_else(|| PgAdapterError::DoesNotExist("User".to_string()).into())
    }

    /// Sets the user's frozen flag to true
    fn freeze(&self, user_id: &str) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(frozen.eq(true))
            .load::<User>(&mut self.client.connect()?)?
            .pop()
            .ok_or_else(|| PgAdapterError::DoesNotExist("User".to_string()).into())
    }

    /// Returns the total count of users and a vec of users constrained by the options as
    /// the first and second element respectively
    fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<SortOptions>,
    ) -> Result<Vec<User>, AdapterError> {
        use super::schema::users::dsl::*;
        let mut query = users.into_boxed();

        if let Some(sort) = sort {
            match sort {
                SortOptions::UsernameAsc => query = query.order(username.asc()),
                SortOptions::UsernameDesc => query = query.order(username.desc()),
                SortOptions::EmailAsc => query = query.order(email.asc()),
                SortOptions::EmailDesc => query = query.order(email.desc()),
                SortOptions::CreatedAtAsc => query = query.order(created_at.asc()),
                SortOptions::CreatedAtDesc => query = query.order(created_at.desc()),
            };
        }

        query = query.offset(((page - 1) * per_page) as i64);
        query = query.limit(per_page as i64);

        let result = query.load::<User>(&mut self.client.connect()?)?;

        Ok(result)
    }
}
