use super::{schema::users, PgAdapterError};
use crate::{
    clients::store::postgres::Postgres,
    store::repository::user::{SortOptions, User, UserRepository},
};
use async_trait::async_trait;
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

#[async_trait]
impl UserRepository for PgUserAdapter {
    type Error = PgAdapterError;

    async fn create(
        &self,
        user_email: &str,
        user_name: &str,
        user_pw: &str,
    ) -> Result<User, Self::Error> {
        use super::schema::users::dsl::*;
        diesel::insert_into(users)
            .values(NewUser {
                email: user_email,
                username: user_name,
                password: user_pw,
            })
            .get_result::<User>(&mut self.client.connect()?)
            .map_err(Self::Error::new)
    }

    /// Fetches a user by their ID
    async fn get_by_id(&self, user_id: &str) -> Result<User, Self::Error> {
        use super::schema::users::dsl::*;
        users
            .filter(id.eq(user_id))
            .first::<User>(&mut self.client.connect()?)
            .map_err(Self::Error::new)
    }

    /// Fetches a user by their email
    async fn get_by_email(&self, user_email: &str) -> Result<User, Self::Error> {
        use super::schema::users::dsl::*;
        users
            .filter(email.eq(user_email))
            .first::<User>(&mut self.client.connect()?)
            .map_err(Self::Error::new)
    }

    /// Hashes the given password with bcrypt and sets the user's password field to the hash
    async fn update_password(&self, user_id: &str, pw_hash: &str) -> Result<User, Self::Error> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(password.eq(pw_hash))
            .load::<User>(&mut self.client.connect()?)
            .map_err(Self::Error::new)?
            .pop()
            .ok_or_else(|| PgAdapterError::DoesNotExist("User".to_string()))
    }

    /// Sets the user's frozen flag to true
    async fn update_email_verified_at(&self, user_id: &str) -> Result<User, Self::Error> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(email_verified_at.eq(chrono::Utc::now()))
            .load::<User>(&mut self.client.connect()?)
            .map_err(Self::Error::new)?
            .pop()
            .ok_or_else(|| PgAdapterError::DoesNotExist("User".to_string()))
    }

    /// Updates the user's OTP secret to the given key
    async fn update_otp_secret(&self, user_id: &str, secret: &str) -> Result<User, Self::Error> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(otp_secret.eq(Some(secret)))
            .load::<User>(&mut self.client.connect()?)
            .map_err(Self::Error::new)?
            .pop()
            .ok_or_else(|| PgAdapterError::DoesNotExist("User".to_string()))
    }

    /// Sets the user's frozen flag to true
    async fn freeze(&self, user_id: &str) -> Result<User, Self::Error> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(frozen.eq(true))
            .load::<User>(&mut self.client.connect()?)
            .map_err(Self::Error::new)?
            .pop()
            .ok_or_else(|| PgAdapterError::DoesNotExist("User".to_string()))
    }

    /// Returns the total count of users and a vec of users constrained by the options as
    /// the first and second element respectively
    async fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort: Option<SortOptions>,
    ) -> Result<Vec<User>, Self::Error> {
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
