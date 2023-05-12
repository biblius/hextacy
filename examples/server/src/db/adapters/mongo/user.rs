use crate::{
    db::{
        models::{
            role::Role,
            user::{SortOptions, User},
        },
        repository::user::UserRepository,
        RepoAdapterError,
    },
    services::oauth::OAuthProvider,
};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use mongodb::{bson::doc, ClientSession};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct MgUserAdapter;

#[async_trait]
impl UserRepository<ClientSession> for MgUserAdapter {
    async fn create(
        conn: &mut ClientSession,
        email: &str,
        username: &str,
        password: &str,
    ) -> Result<User, RepoAdapterError> {
        let db = conn.client().database("xtc");
        let user = NewUser::from_initial(email, username, password);
        let _id = db
            .collection("users")
            .insert_one_with_session(user, None, conn)
            .await
            .unwrap()
            .inserted_id;

        let user = db
            .collection("users")
            .find_one(doc! {"_id": _id}, None)
            .await?
            .unwrap();

        Ok(user)
    }

    async fn create_from_oauth(
        conn: &mut ClientSession,
        account_id: &str,
        email: &str,
        username: &str,
        provider: OAuthProvider,
    ) -> Result<User, RepoAdapterError> {
        let db = conn.client().database("xtc");

        let mut user = NewUser {
            email,
            username,
            ..Default::default()
        };

        user.set_provider_id(account_id, provider);

        let id = db
            .collection("users")
            .insert_one_with_session(user, None, conn)
            .await
            .unwrap()
            .inserted_id;

        let user = db
            .collection("users")
            .find_one(doc! {"id": id}, None)
            .await?
            .unwrap();

        Ok(user)
    }

    async fn get_by_id(conn: &mut ClientSession, id: &str) -> Result<User, RepoAdapterError> {
        let db = conn.client().database("xtc");
        let user = db
            .collection("users")
            .find_one(doc! {"id": id}, None)
            .await?
            .unwrap();

        Ok(user)
    }

    async fn get_by_oauth_id(
        conn: &mut ClientSession,
        id: &str,
        provider: OAuthProvider,
    ) -> Result<User, RepoAdapterError> {
        let db = conn.client().database("xtc");

        let provider_id = match provider {
            OAuthProvider::Google => "google_id",
            OAuthProvider::Github => "github_id",
        };

        let user = db
            .collection("users")
            .find_one(doc! {provider_id: id}, None)
            .await?
            .unwrap();

        Ok(user)
    }

    async fn get_by_email(conn: &mut ClientSession, email: &str) -> Result<User, RepoAdapterError> {
        let db = conn.client().database("xtc");
        db.collection("users")
            .find_one(doc! {"email": email}, None)
            .await?
            .ok_or_else(|| RepoAdapterError::DoesNotExist)
    }

    async fn update_password(
        conn: &mut ClientSession,
        id: &str,
        password: &str,
    ) -> Result<User, RepoAdapterError> {
        let db = conn.client().database("xtc");

        let _id = db
            .collection::<User>("users")
            .update_one_with_session(
                doc! {"id": id},
                doc! {"$set" : {"password" : password}},
                None,
                conn,
            )
            .await?
            .upserted_id;

        let user = db
            .collection("users")
            .find_one(doc! {"_id": _id}, None)
            .await?
            .unwrap();

        Ok(user)
    }

    async fn update_otp_secret(
        conn: &mut ClientSession,
        id: &str,
        secret: &str,
    ) -> Result<User, RepoAdapterError> {
        let db = conn.client().database("xtc");
        let _id = db
            .collection::<User>("users")
            .update_one_with_session(
                doc! {"id": id},
                doc! {"$set" : {"otp_secret" : secret}},
                None,
                conn,
            )
            .await?
            .upserted_id;

        let user = db
            .collection("users")
            .find_one(doc! {"_id": _id}, None)
            .await?
            .unwrap();

        Ok(user)
    }

    async fn update_email_verified_at(
        conn: &mut ClientSession,
        id: &str,
    ) -> Result<User, RepoAdapterError> {
        let db = conn.client().database("xtc");
        let _id = db
            .collection::<User>("users")
            .update_one_with_session(
                doc! {"id": id},
                doc! {"$set" : {"email_verified_at" : chrono::Utc::now().to_string()}},
                None,
                conn,
            )
            .await?
            .upserted_id;

        let user = db
            .collection("users")
            .find_one(doc! {"_id": _id}, None)
            .await?
            .unwrap();

        Ok(user)
    }

    async fn update_oauth_id(
        conn: &mut ClientSession,
        id: &str,
        oauth_id: &str,
        provider: OAuthProvider,
    ) -> Result<User, RepoAdapterError> {
        let db = conn.client().database("xtc");

        let provider_id = match provider {
            OAuthProvider::Google => "google_id",
            OAuthProvider::Github => "github_id",
        };

        let _id = db
            .collection::<User>("users")
            .update_one_with_session(
                doc! {"id": id},
                doc! {"$set" : {provider_id : oauth_id}},
                None,
                conn,
            )
            .await?
            .upserted_id;

        let user = db
            .collection("users")
            .find_one(doc! {"_id": _id}, None)
            .await?
            .unwrap();

        Ok(user)
    }

    async fn freeze(conn: &mut ClientSession, id: &str) -> Result<User, RepoAdapterError> {
        let db = conn.client().database("xtc");
        let _id = db
            .collection::<User>("users")
            .update_one_with_session(
                doc! {"id": id},
                doc! {"$set" : {"frozen" : true}},
                None,
                conn,
            )
            .await?
            .upserted_id;

        let user = db
            .collection("users")
            .find_one(doc! {"_id": _id}, None)
            .await?
            .unwrap();

        Ok(user)
    }

    async fn get_paginated(
        conn: &mut ClientSession,
        _page: u16,
        _per_page: u16,
        _sort_by: Option<SortOptions>,
    ) -> Result<Vec<User>, RepoAdapterError> {
        let _db = conn.client().database("xtc");
        todo!()
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct NewUser<'a> {
    id: String,
    email: &'a str,
    username: &'a str,
    first_name: Option<&'a str>,
    last_name: Option<&'a str>,
    role: Role,
    phone: Option<&'a str>,
    password: Option<&'a str>,
    otp_secret: Option<&'a str>,
    frozen: bool,
    google_id: Option<&'a str>,
    github_id: Option<&'a str>,
    email_verified_at: Option<NaiveDateTime>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl<'a> NewUser<'a> {
    fn from_initial(email: &'a str, username: &'a str, password: &'a str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            email,
            username,
            password: Some(password),
            role: Role::User,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
            ..Default::default()
        }
    }

    fn set_provider_id(&mut self, id: &'a str, provider: OAuthProvider) {
        match provider {
            OAuthProvider::Google => self.google_id = Some(id),
            OAuthProvider::Github => self.github_id = Some(id),
        }
    }
}

#[derive(Debug, Default, PartialEq)]
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
