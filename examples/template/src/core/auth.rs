use super::models::user::User;
use crate::{
    core::{
        models::session::Session,
        repository::{session::SessionRepository, user::UserRepository},
    },
    error::Error,
    AppResult,
};
use hextacy::{queue::Producer, Driver};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct UserRegisteredEvent {
    id: Uuid,
    username: String,
}

#[derive(Debug, Clone)]
pub struct Authentication<U, S, P> {
    pub user_repo: U,
    pub session_repo: S,
    pub producer: P,
}

impl<U, S, P> Authentication<U, S, P>
where
    U: UserRepository,
    S: SessionRepository,
    P: Producer,
{
    pub async fn register(&self, username: &str, password: &str) -> AppResult<(User, Session)> {
        match self.user_repo.get_by_username(username).await {
            Ok(None) => {}
            Ok(Some(_)) => return Err(AuthenticationError::UsernameTaken.into()),
            Err(e) => return Err(e.into()),
        };

        let hashed = hextacy::crypto::bcrypt_hash(password, 10)?;

        /*         let (user, session) = transaction!(
            conn: R => {
                let user = self.user_repo.create(&mut conn, username, &hashed).await?;
                let session = self.session_repo.create(&user, true).await?;
                self.producer
                    .publish(UserRegisteredEvent {
                      id: user.id,
                      username: user.username.clone(),
                })
                .await?;
                Ok((user, session))
            }
        )?;

        Ok((user, session)) */

        todo!()
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
        remember: bool,
    ) -> AppResult<Session> {
        let user = match self.user_repo.get_by_username(username).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(AuthenticationError::InvalidCredentials.into()),
            Err(e) => return Err(e.into()),
        };

        let valid = hextacy::crypto::bcrypt_verify(password, &user.password)?;
        if !valid {
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        let session = self.session_repo.create(&user, !remember).await?;

        Ok(session)
    }

    pub async fn logout(&self, session_id: Uuid, purge: bool) -> AppResult<u64> {
        let session = self.session_repo.expire(session_id).await?;
        if purge {
            return self
                .session_repo
                .purge(session.user_id)
                .await
                .map_err(Error::new);
        }
        Ok(1)
    }
}

#[derive(Debug, Error, Serialize)]
pub enum AuthenticationError {
    #[error("Unauthenticated")]
    Unauthenticated,
    #[error("Username taken")]
    UsernameTaken,
    #[error("Invalid credentials")]
    InvalidCredentials,
}
