#![allow(unused_variables)]
use super::entity::users::{ActiveModel, Column, Entity};
use crate::db::{
    adapters::AdapterError,
    models::user::{SortOptions, User},
    repository::user::UserRepository,
};
use crate::services::oauth::OAuthProvider;
use async_trait::async_trait;
use sea_orm::{prelude::*, DatabaseTransaction, Set};
use sea_orm::{query::*, DatabaseConnection, QuerySelect};

#[derive(Debug, Clone)]
pub struct PgUserAdapter;

#[async_trait]
impl UserRepository<DatabaseConnection> for PgUserAdapter {
    async fn create(
        conn: &mut DatabaseConnection,
        email: &str,
        username: &str,
        password: &str,
    ) -> Result<User, AdapterError> {
        let user = ActiveModel {
            email: Set(email.to_string()),
            username: Set(username.to_string()),
            password: Set(Some(password.to_string())),
            ..Default::default()
        };
        user.insert(conn)
            .await
            .map(User::from)
            .map_err(AdapterError::from)
    }

    async fn create_from_oauth(
        conn: &mut DatabaseConnection,
        account_id: &str,
        email: &str,
        username: &str,
        provider: OAuthProvider,
    ) -> Result<User, AdapterError> {
        let (github, google) = match provider {
            OAuthProvider::Github => (Some(account_id.to_string()), None),
            OAuthProvider::Google => (None, Some(account_id.to_string())),
        };
        let user = ActiveModel {
            email: Set(email.to_string()),
            username: Set(username.to_string()),
            google_id: Set(google),
            github_id: Set(github),
            ..Default::default()
        };
        user.insert(conn)
            .await
            .map(User::from)
            .map_err(AdapterError::from)
    }

    /// Fetches a user by their ID
    async fn get_by_id(conn: &mut DatabaseConnection, user_id: &str) -> Result<User, AdapterError> {
        todo!()
    }

    async fn get_by_oauth_id(
        conn: &mut DatabaseConnection,
        oauth_id: &str,
        provider: OAuthProvider,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    /// Fetches a user by their email
    async fn get_by_email(
        conn: &mut DatabaseConnection,
        user_email: &str,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    /// Hashes the given password with bcrypt and sets the user's password field to the hash
    async fn update_password(
        conn: &mut DatabaseConnection,
        user_id: &str,
        pw_hash: &str,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    /// Updates the user's OTP secret to the given key
    async fn update_otp_secret(
        conn: &mut DatabaseConnection,
        user_id: &str,
        secret: &str,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    /// Update the user's email verified at field to now
    async fn update_email_verified_at(
        conn: &mut DatabaseConnection,
        user_id: &str,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    async fn update_oauth_id(
        conn: &mut DatabaseConnection,
        id: &str,
        oauth_id: &str,
        provider: OAuthProvider,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    /// Sets the user's frozen flag to true
    async fn freeze(conn: &mut DatabaseConnection, user_id: &str) -> Result<User, AdapterError> {
        todo!()
    }

    /// Returns the total count of users and a vec of users constrained by the options as
    /// the first and second element respectively
    async fn get_paginated(
        conn: &mut DatabaseConnection,
        page: u16,
        per_page: u16,
        sort: Option<SortOptions>,
    ) -> Result<Vec<User>, AdapterError> {
        let mut query = Entity::find()
            .offset(((page - 1) * per_page) as u64)
            .limit(per_page as u64);

        if let Some((col, ord)) = sort.map(sort_transform) {
            query = query.order_by(col, ord);
        }

        query
            .all(conn)
            .await
            .map(|res| res.into_iter().map(User::from).collect())
            .map_err(AdapterError::from)
    }
}

fn sort_transform(sort: SortOptions) -> (Column, Order) {
    use Column::*;
    use Order::*;
    match sort {
        SortOptions::UsernameAsc => (Username, Asc),
        SortOptions::UsernameDesc => (Username, Desc),
        SortOptions::EmailAsc => (Email, Asc),
        SortOptions::EmailDesc => (Email, Desc),
        SortOptions::CreatedAtAsc => (CreatedAt, Asc),
        SortOptions::CreatedAtDesc => (CreatedAt, Desc),
    }
}

#[async_trait]
impl UserRepository<DatabaseTransaction> for PgUserAdapter {
    async fn create(
        conn: &mut DatabaseTransaction,
        email: &str,
        username: &str,
        password: &str,
    ) -> Result<User, AdapterError> {
        let user = ActiveModel {
            email: Set(email.to_string()),
            username: Set(username.to_string()),
            password: Set(Some(password.to_string())),
            ..Default::default()
        };
        user.insert(conn)
            .await
            .map(User::from)
            .map_err(AdapterError::from)
    }

    async fn create_from_oauth(
        conn: &mut DatabaseTransaction,
        account_id: &str,
        email: &str,
        username: &str,
        provider: OAuthProvider,
    ) -> Result<User, AdapterError> {
        let (github, google) = match provider {
            OAuthProvider::Github => (Some(account_id.to_string()), None),
            OAuthProvider::Google => (None, Some(account_id.to_string())),
        };
        let user = ActiveModel {
            email: Set(email.to_string()),
            username: Set(username.to_string()),
            google_id: Set(google),
            github_id: Set(github),
            ..Default::default()
        };
        user.insert(conn)
            .await
            .map(User::from)
            .map_err(AdapterError::from)
    }

    /// Fetches a user by their ID
    async fn get_by_id(
        conn: &mut DatabaseTransaction,
        user_id: &str,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    async fn get_by_oauth_id(
        conn: &mut DatabaseTransaction,
        oauth_id: &str,
        provider: OAuthProvider,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    /// Fetches a user by their email
    async fn get_by_email(
        conn: &mut DatabaseTransaction,
        user_email: &str,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    /// Hashes the given password with bcrypt and sets the user's password field to the hash
    async fn update_password(
        conn: &mut DatabaseTransaction,
        user_id: &str,
        pw_hash: &str,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    /// Updates the user's OTP secret to the given key
    async fn update_otp_secret(
        conn: &mut DatabaseTransaction,
        user_id: &str,
        secret: &str,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    /// Update the user's email verified at field to now
    async fn update_email_verified_at(
        conn: &mut DatabaseTransaction,
        user_id: &str,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    async fn update_oauth_id(
        conn: &mut DatabaseTransaction,
        id: &str,
        oauth_id: &str,
        provider: OAuthProvider,
    ) -> Result<User, AdapterError> {
        todo!()
    }

    /// Sets the user's frozen flag to true
    async fn freeze(conn: &mut DatabaseTransaction, user_id: &str) -> Result<User, AdapterError> {
        todo!()
    }

    /// Returns the total count of users and a vec of users constrained by the options as
    /// the first and second element respectively
    async fn get_paginated(
        conn: &mut DatabaseTransaction,
        page: u16,
        per_page: u16,
        sort: Option<SortOptions>,
    ) -> Result<Vec<User>, AdapterError> {
        let mut query = Entity::find()
            .offset(((page - 1) * per_page) as u64)
            .limit(per_page as u64);

        if let Some((col, ord)) = sort.map(sort_transform) {
            query = query.order_by(col, ord);
        }

        query
            .all(conn)
            .await
            .map(|res| res.into_iter().map(User::from).collect())
            .map_err(AdapterError::from)
    }
}
