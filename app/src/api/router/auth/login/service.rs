use super::data::{AuthenticationResponse, AuthenticationSuccess, Credentials, Otp, Prompt2FA};
use crate::{
    api::services::cache::{Cache, CachePrefix},
    error::{AuthenticationError, Error},
    models::{session::Session, user::User},
};
use actix_web::{HttpResponse, Responder};
use infrastructure::{
    config::constants::{CSRF_CACHE_DURATION_SECONDS, SESSION_CACHE_DURATION_SECONDS},
    crypto::{
        self,
        utils::{bcrypt_verify, generate_hmac, generate_hmac_random},
    },
    http::cookie,
    storage::{
        postgres::Pg,
        redis::{Commands, Rd},
    },
};
use std::sync::Arc;

pub struct Authentication {
    pub pg: PgService,
    pub cache: CacheService,
}

impl Authentication {
    pub async fn verify_credentials(
        &self,
        credentials: Credentials<'_>,
    ) -> Result<HttpResponse, Error> {
        let (email, password) = credentials.data();

        let user = match self.pg.find_user_by_email(email).await {
            Ok(u) => u,
            Err(_) => return Err(AuthenticationError::InvalidCredentials.into()),
        };

        if user.email_verified_at.is_none() {
            return Err(AuthenticationError::UnverifiedEmail.into());
        }

        if user.frozen {
            return Err(AuthenticationError::AccountFrozen.into());
        }

        match user.password {
            Some(ref pw) => {
                if !bcrypt_verify(password, pw)? {
                    return Err(AuthenticationError::InvalidCredentials.into());
                }
            }
            _ => return Err(AuthenticationError::InvalidCredentials.into()),
        }

        // If the user has 2FA turned on stop and cache the user so we can quickly verify their otp
        if user.otp_secret.is_some() {
            let temp = generate_hmac_random("HMAC_SECRET")?;
            self.cache.set_user(CachePrefix::TempOtp, &temp, &user);
            return Ok(Prompt2FA::new(user.username, temp).to_response(None));
        }

        // Create and cache the session
        let session = self.pg.create_session(&user).await?;
        let csrf_token = generate_hmac("CSRF_SECRET", &session.id)?;

        let session_cookie = cookie::create("session", &session, None)?;
        let csrf_cookie = cookie::csrf(&csrf_token);

        self.cache.set_session(&csrf_token, &session).await?;

        Ok(AuthenticationSuccess::new(user, session)
            .to_response(Some(vec![session_cookie, csrf_cookie])))
    }

    pub async fn verify_otp(&self, otp: Otp<'_>) -> Result<impl Responder, Error> {
        let (password, token) = otp.data();

        let user = self.cache.get_user(CachePrefix::TempOtp, token).await?;

        if let Some(ref secret) = user.otp_secret {
            let (result, _) = crypto::utils::verify_otp(password, &secret)?;

            if !result {
                return Err(AuthenticationError::InvalidOTP.into());
            }

            let session = self.pg.create_session(&user).await?;
            let csrf_token = generate_hmac("CSRF_SECRET", &session.id)?;
            let session_cookie = cookie::create("session", &session, None)?;
            let csrf_cookie = cookie::csrf(&csrf_token);

            Ok(AuthenticationSuccess::new(user, session)
                .to_response(Some(vec![session_cookie, csrf_cookie])))
        } else {
            Err(AuthenticationError::InvalidOTP.into())
        }
    }
}

pub struct PgService {
    pg_pool: Arc<Pg>,
}

impl PgService {
    async fn find_user_by_email(&self, email: &str) -> Result<User, Error> {
        User::get_by_email(email, &mut self.pg_pool.connect()?)
    }
    async fn create_session(&self, user: &User) -> Result<Session, Error> {
        Session::create(user, &mut self.pg_pool.connect()?)
    }
}

pub struct CacheService {
    rd_pool: Arc<Rd>,
}

impl CacheService {
    async fn set_session(&self, token: &str, session: &Session) -> Result<(), Error> {
        let mut connection = self.rd_pool.connect()?;

        let json = session.to_json()?;

        connection.set_ex::<&str, String, ()>(&token, json, SESSION_CACHE_DURATION_SECONDS);
        Ok(())
    }

    async fn set_user(&self, prefix: CachePrefix, token: &str, user: &User) -> Result<(), Error> {
        let mut connection = self.rd_pool.connect()?;

        Cache::set(
            prefix,
            &token,
            user,
            Some(CSRF_CACHE_DURATION_SECONDS),
            &mut connection,
        )?;

        Ok(())
    }

    async fn get_user(&self, prefix: CachePrefix, token: &str) -> Result<User, Error> {
        let mut connection = self.rd_pool.connect()?;

        Cache::get(prefix, token, &mut connection)
    }
}
