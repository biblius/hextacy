use super::models::user::User;
use crate::{
    cache::contracts::BasicCacheAccess,
    core::{
        models::session::Session,
        repository::{session::SessionRepository, user::UserRepository},
    },
    error::Error,
    AppResult,
};
use hextacy::{component, queue::Producer, transaction};
use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct UserRegisteredEvent {
    id: Uuid,
    username: String,
}

#[component(
    use Repo as repo,
    use Cache as cache,

    use UserRepo, SessionRepo, CacheAccess, Publisher
)]
#[derive(Debug, Clone)]
pub struct Authentication {}

#[component(
    use Repo:Atomic for
        UR: UserRepository,
        SR: SessionRepository,

    use Cache for
        CA: BasicCacheAccess
)]
impl<P> Authentication<P>
where
    P: Producer,
{
    pub async fn register(&self, username: &str, password: &str) -> AppResult<(User, Session)> {
        let mut conn = self.repo.connect().await?;

        match self.user_repo.get_by_username(&mut conn, username).await {
            Ok(None) => {}
            Ok(Some(_)) => return Err(AuthenticationError::UsernameTaken.into()),
            Err(e) => return Err(e.into()),
        };

        let hashed = hextacy::crypto::bcrypt_hash(password, 10)?;

        let (user, session) = transaction!(
            conn: Repo => {
                let user = self.user_repo.create(&mut conn, username, &hashed).await?;
                let session = self.session_repo.create(&mut conn, &user, true).await?;
                self.publisher
                    .publish(UserRegisteredEvent {
                      id: user.id,
                      username: user.username.clone(),
                })
                .await?;
                Ok((user, session))
            }
        )?;

        Ok((user, session))
    }

    pub async fn login(
        &self,
        username: &str,
        password: &str,
        remember: bool,
    ) -> AppResult<Session> {
        let mut conn = self.repo.connect().await?;

        let user = match self.user_repo.get_by_username(&mut conn, username).await {
            Ok(Some(user)) => user,
            Ok(None) => return Err(AuthenticationError::InvalidCredentials.into()),
            Err(e) => return Err(e.into()),
        };

        let valid = hextacy::crypto::bcrypt_verify(password, &user.password)?;
        if !valid {
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        let session = self
            .session_repo
            .create(&mut conn, &user, !remember)
            .await?;

        Ok(session)
    }

    pub async fn logout(&self, session_id: Uuid, purge: bool) -> AppResult<u64> {
        let mut conn = self.repo.connect().await?;
        let session = self.session_repo.expire(&mut conn, session_id).await?;
        if purge {
            return self
                .session_repo
                .purge(&mut conn, session.user_id)
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
