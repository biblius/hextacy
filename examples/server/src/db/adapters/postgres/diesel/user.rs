use crate::db::adapters::postgres::diesel::schema::users;
use crate::db::models::role::Role;
use crate::db::{
    models::user::{SortOptions, User},
    repository::user::UserRepository,
    RepoAdapterError,
};
use crate::services::oauth::OAuthProvider;
use async_trait::async_trait;
use diesel::{AsChangeset, ExpressionMethods, Insertable, QueryDsl, RunQueryDsl};
use hextacy::drivers::db::postgres::diesel::DieselConnection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PgUserAdapter;

#[async_trait]
impl UserRepository<DieselConnection> for PgUserAdapter {
    async fn create(
        conn: &mut DieselConnection,
        email: &str,
        username: &str,
        password: &str,
    ) -> Result<User, RepoAdapterError> {
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
            .get_result::<User>(conn)
            .map_err(|e| e.into())
    }

    async fn create_from_oauth(
        conn: &mut DieselConnection,
        account_id: &str,
        email: &str,
        username: &str,
        provider: OAuthProvider,
    ) -> Result<User, RepoAdapterError> {
        use super::schema::users::dsl;

        let mut user = NewUser {
            email,
            username,
            password: None,
            github_id: None,
            google_id: None,
        };

        user.set_provider_id(account_id, provider);

        diesel::insert_into(dsl::users)
            .values(user)
            .get_result::<User>(conn)
            .map_err(|e| e.into())
    }

    /// Fetches a user by their ID
    async fn get_by_id(
        conn: &mut DieselConnection,
        user_id: &str,
    ) -> Result<User, RepoAdapterError> {
        use super::schema::users::dsl::*;
        users
            .filter(id.eq(user_id))
            .first::<User>(conn)
            .map_err(|e| e.into())
    }

    async fn get_by_oauth_id(
        conn: &mut DieselConnection,
        oauth_id: &str,
        provider: OAuthProvider,
    ) -> Result<User, RepoAdapterError> {
        use super::schema::users::dsl::*;

        let mut query = users.into_boxed();

        match provider {
            OAuthProvider::Google => query = query.filter(google_id.eq(oauth_id)),
            OAuthProvider::Github => query = query.filter(github_id.eq(oauth_id)),
        };

        query.first::<User>(conn).map_err(|e| e.into())
    }

    /// Fetches a user by their email
    async fn get_by_email(
        conn: &mut DieselConnection,
        user_email: &str,
    ) -> Result<User, RepoAdapterError> {
        use super::schema::users::dsl::*;
        users
            .filter(email.eq(user_email))
            .first::<User>(conn)
            .map_err(|e| e.into())
    }

    /// Hashes the given password with bcrypt and sets the user's password field to the hash
    async fn update_password(
        conn: &mut DieselConnection,
        user_id: &str,
        pw_hash: &str,
    ) -> Result<User, RepoAdapterError> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(password.eq(pw_hash))
            .load::<User>(conn)?
            .pop()
            .ok_or_else(|| RepoAdapterError::DoesNotExist)
    }

    /// Updates the user's OTP secret to the given key
    async fn update_otp_secret(
        conn: &mut DieselConnection,
        user_id: &str,
        secret: &str,
    ) -> Result<User, RepoAdapterError> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(otp_secret.eq(Some(secret)))
            .load::<User>(conn)?
            .pop()
            .ok_or_else(|| RepoAdapterError::DoesNotExist)
    }

    /// Update the user's email verified at field to now
    async fn update_email_verified_at(
        conn: &mut DieselConnection,
        user_id: &str,
    ) -> Result<User, RepoAdapterError> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(email_verified_at.eq(chrono::Utc::now()))
            .load::<User>(conn)?
            .pop()
            .ok_or_else(|| RepoAdapterError::DoesNotExist)
    }

    async fn update_oauth_id(
        conn: &mut DieselConnection,
        id: &str,
        oauth_id: &str,
        provider: OAuthProvider,
    ) -> Result<User, RepoAdapterError> {
        use super::schema::users::dsl;

        let mut update = UserUpdate {
            google_id: None,
            github_id: None,
            ..Default::default()
        };

        match provider {
            OAuthProvider::Google => update.google_id = Some(oauth_id),
            OAuthProvider::Github => update.github_id = Some(oauth_id),
        };

        diesel::update(dsl::users)
            .filter(dsl::id.eq(id))
            .set(update)
            .load::<User>(conn)?
            .pop()
            .ok_or_else(|| RepoAdapterError::DoesNotExist)
    }

    /// Sets the user's frozen flag to true
    async fn freeze(conn: &mut DieselConnection, user_id: &str) -> Result<User, RepoAdapterError> {
        use super::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user_id)))
            .set(frozen.eq(true))
            .load::<User>(conn)?
            .pop()
            .ok_or_else(|| RepoAdapterError::DoesNotExist)
    }

    /// Returns the total count of users and a vec of users constrained by the options as
    /// the first and second element respectively
    async fn get_paginated(
        conn: &mut DieselConnection,
        page: u16,
        per_page: u16,
        sort: Option<SortOptions>,
    ) -> Result<Vec<User>, RepoAdapterError> {
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

        let result = query.load::<User>(conn)?;

        Ok(result)
    }
}

#[derive(Debug, Deserialize, Serialize, Insertable)]
#[diesel(table_name = users)]
struct NewUser<'a> {
    email: &'a str,
    username: &'a str,
    password: Option<&'a str>,
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

#[derive(Debug, Default, PartialEq, AsChangeset)]
#[diesel(table_name = users)]
struct UserUpdate<'a> {
    email: Option<&'a str>,
    username: Option<&'a str>,
    first_name: Option<&'a str>,
    last_name: Option<&'a str>,
    role: Option<&'a Role>,
    phone: Option<&'a str>,
    password: Option<&'a str>,
    otp_secret: Option<&'a str>,
    google_id: Option<&'a str>,
    pub github_id: Option<&'a str>,
}
