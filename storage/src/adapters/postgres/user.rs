use super::schema::users;
use crate::{
    adapters::AdapterError,
    models::user::{SortOptions, User},
    repository::user::UserRepository,
    Transaction,
};
use alx_clients::{db::postgres::Postgres, oauth::OAuthProvider};
use diesel::{AsChangeset, ExpressionMethods, Insertable, PgConnection, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize, Insertable)]
#[diesel(table_name = users)]
struct NewUser<'a> {
    email: &'a str,
    username: &'a str,
    password: Option<&'a str>,
    google_id: Option<&'a str>,
    github_id: Option<&'a str>,
}

#[derive(Debug, Deserialize, Serialize, AsChangeset)]
#[diesel(table_name = users)]
pub struct UserUpdate<'a> {
    google_id: Option<&'a str>,
    github_id: Option<&'a str>,
}

impl<'a> NewUser<'a> {
    fn set_provider_id(&mut self, id: &'a str, provider: OAuthProvider) {
        match provider {
            OAuthProvider::Google => self.google_id = Some(id),
            OAuthProvider::Github => self.github_id = Some(id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PgUserAdapter {
    pub client: Arc<Postgres>,
}

impl UserRepository<PgConnection> for PgUserAdapter {
    fn create(&self, email: &str, username: &str, password: &str) -> Result<User, AdapterError> {
        use super::schema::users::dsl;

        let user = NewUser {
            email,
            username,
            password: Some(password),
            google_id: None,
            github_id: None,
        };

        diesel::insert_into(dsl::users)
            .values(user)
            .get_result::<User>(&mut self.client.connect()?)
            .map_err(|e| e.into())
    }

    fn create_from_oauth(
        &self,
        account_id: &str,
        email: &str,
        username: &str,
        provider: OAuthProvider,
        mut trx: Option<&mut Transaction<PgConnection>>,
    ) -> Result<User, AdapterError> {
        use super::schema::users::dsl;

        let mut user = NewUser {
            email,
            username,
            password: None,
            google_id: None,
            github_id: None,
        };

        user.set_provider_id(account_id, provider);
        let transaction = trx.as_mut().map(|t| t.inner());

        let insert = diesel::insert_into(dsl::users).values(user);

        match transaction {
            Some(trx) => insert.get_result::<User>(trx).map_err(|e| e.into()),
            None => insert
                .get_result::<User>(&mut self.client.connect()?)
                .map_err(|e| e.into()),
        }
    }

    /// Fetches a user by their ID
    fn get_by_id(&self, user_id: &str) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;
        users
            .filter(id.eq(user_id))
            .first::<User>(&mut self.client.connect()?)
            .map_err(|e| e.into())
    }

    fn get_by_oauth_id(
        &self,
        oauth_id: &str,
        provider: OAuthProvider,
    ) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;

        let mut query = users.into_boxed();

        match provider {
            alx_clients::oauth::OAuthProvider::Google => {
                query = query.filter(google_id.eq(oauth_id))
            }
            alx_clients::oauth::OAuthProvider::Github => {
                query = query.filter(github_id.eq(oauth_id))
            }
        };

        query
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
            .ok_or_else(|| AdapterError::DoesNotExist)
    }

    /// Updates the user's OTP secret to the given key
    fn update_otp_secret(&self, user_id: &str, secret: &str) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(otp_secret.eq(Some(secret)))
            .load::<User>(&mut self.client.connect()?)?
            .pop()
            .ok_or_else(|| AdapterError::DoesNotExist)
    }

    /// Update the user's email verified at field to now
    fn update_email_verified_at(&self, user_id: &str) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(email_verified_at.eq(chrono::Utc::now()))
            .load::<User>(&mut self.client.connect()?)?
            .pop()
            .ok_or_else(|| AdapterError::DoesNotExist)
    }

    fn update_oauth_id(
        &self,
        id: &str,
        oauth_id: &str,
        provider: OAuthProvider,
    ) -> Result<User, AdapterError> {
        use super::schema::users::dsl;

        let mut update = UserUpdate {
            google_id: None,
            github_id: None,
        };

        match provider {
            OAuthProvider::Google => update.google_id = Some(oauth_id),
            OAuthProvider::Github => update.github_id = Some(oauth_id),
        };

        diesel::update(dsl::users)
            .filter(dsl::id.eq(id))
            .set(update)
            .load::<User>(&mut self.client.connect()?)?
            .pop()
            .ok_or_else(|| AdapterError::DoesNotExist)
    }

    /// Sets the user's frozen flag to true
    fn freeze(&self, user_id: &str) -> Result<User, AdapterError> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(frozen.eq(true))
            .load::<User>(&mut self.client.connect()?)?
            .pop()
            .ok_or_else(|| AdapterError::DoesNotExist)
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
