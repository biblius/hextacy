use super::{cache::Cache, email::Email, postgres::Postgres};
use crate::{
    api::router::auth::{
        data::{
            login::{Credentials, Otp},
            registration::{RegistrationData, SetPassword},
        },
        response::{
            login::{AuthenticationSuccess, FreezeAccount, Prompt2FA},
            registration::{RegistrationSuccess, TokenVerified},
        },
    },
    error::{AuthenticationError, Error},
    models::user::User,
};
use actix_web::HttpResponse;
use infrastructure::{
    config::constants::{
        MAXIMUM_LOGIN_ATTEMPTS, OTP_TOKEN_DURATION_SECONDS, PW_TOKEN_DURATION_SECONDS,
        REGISTRATION_TOKEN_EXPIRATION_SECONDS,
    },
    crypto::{
        self,
        utils::{bcrypt_verify, generate_hmac},
    },
    http::{cookie, response::Response},
    storage::redis::CacheId,
};
use reqwest::StatusCode;
use tracing::info;

pub(crate) struct Authentication {
    pub database: Postgres,
    pub cache: Cache,
    pub email: Email,
}

// #[async_trait]
impl Authentication {
    pub(crate) async fn verify_credentials(
        &self,
        credentials: Credentials,
    ) -> Result<HttpResponse, Error> {
        let (email, password) = credentials.data();

        let user = match self.database.find_user_by_email(email).await {
            Ok(u) => u,
            Err(_) => return Err(AuthenticationError::InvalidCredentials.into()),
        };

        if user.email_verified_at.is_none() || user.password.is_none() {
            return Err(AuthenticationError::UnverifiedEmail.into());
        }

        if user.frozen {
            return Err(AuthenticationError::AccountFrozen.into());
        }

        // Cache the attempt if it was wrong
        if !bcrypt_verify(password, user.password.as_ref().unwrap())? {
            let attempts = self.cache.cache_login_attempt(&user.id).await?;

            // Freeze the account if attempts exceed the threshold
            if attempts > MAXIMUM_LOGIN_ATTEMPTS as u8 {
                self.database.freeze_user(&user.id).await?;
                return Ok(FreezeAccount::new(
                    &user.id,
                    "Your account has been frozen due to too many invalid login attempts",
                )
                .to_response(StatusCode::LOCKED, None, None));
            }
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        // If the user has 2FA turned on, stop here and cache the user so we can quickly verify their otp
        if user.otp_secret.is_some() {
            let temp = generate_hmac("OTP_HMAC_SECRET", None)?;

            self.cache
                .set_token(
                    CacheId::OTPToken,
                    &temp,
                    &user,
                    Some(OTP_TOKEN_DURATION_SECONDS),
                )
                .await?;

            return Ok(Prompt2FA::new(&user.username, &temp).to_response(
                StatusCode::OK,
                None,
                None,
            ));
        }

        self.generate_session_response(user).await
    }

    /// Verifies the given OTP using the token generated on the credentials login.
    pub(crate) async fn verify_otp(&self, otp: Otp) -> Result<HttpResponse, Error> {
        let (password, token) = otp.data();

        let user = self
            .cache
            .get_token::<User>(CacheId::OTPToken, token)
            .await?;

        if let Some(ref secret) = user.otp_secret {
            let (result, _) = crypto::utils::verify_otp(password, secret)?;

            if !result {
                return Err(AuthenticationError::InvalidOTP.into());
            }

            self.cache.delete_token(CacheId::OTPToken, token).await?;

            self.generate_session_response(user).await
        } else {
            Err(AuthenticationError::InvalidOTP.into())
        }
    }

    /// Stores the initial data in the users table and sends an email to the user with the registration link.
    pub(crate) async fn start_registration(
        &self,
        data: RegistrationData,
    ) -> Result<HttpResponse, Error> {
        let (email, username) = data.inner();

        if self.database.find_user_by_email(email).await.is_ok() {
            return Err(AuthenticationError::EmailTaken.into());
        }

        let user = self.database.create_user(email, username).await?;

        let reg_token = generate_hmac("EMAIL_HMAC_SECRET", Some(&user.id))?;

        self.cache
            .set_token(
                CacheId::RegToken,
                &reg_token,
                &user.id,
                Some(REGISTRATION_TOKEN_EXPIRATION_SECONDS),
            )
            .await?;

        self.email
            .send_registration_token(&reg_token, &user.username, email)?;

        Ok(RegistrationSuccess::new(
            "Successfully sent registration token",
            &user.username,
            &user.email,
        )
        .to_response(StatusCode::CREATED, None, None))
    }

    /// Verifies the registration token sent via email upon registration. Upon success, generates
    /// a one time password token to be used when setting a password.
    pub(crate) async fn verify_registration_token(
        &self,
        token: &str,
    ) -> Result<HttpResponse, Error> {
        let user_id = match self
            .cache
            .get_token::<String>(CacheId::RegToken, token)
            .await
        {
            Ok(id) => id,
            Err(_) => return Err(AuthenticationError::InvalidToken.into()),
        };

        self.database.update_email_verified_at(&user_id).await?;

        self.cache.delete_token(CacheId::RegToken, token).await?;

        let pw_token = generate_hmac("PW_TOKEN_SECRET", None)?;

        self.cache
            .set_token(
                CacheId::PWToken,
                &pw_token,
                &user_id,
                Some(PW_TOKEN_DURATION_SECONDS),
            )
            .await?;

        Ok(
            TokenVerified::new("Successfully verified registration token").to_response(
                StatusCode::OK,
                None,
                None,
            ),
        )
    }

    pub(crate) async fn set_password(
        &self,
        user_identifier: String,
        data: SetPassword,
    ) -> Result<HttpResponse, Error> {
        let (token, password) = data.inner();

        let user_id = match self
            .cache
            .get_token::<String>(CacheId::PWToken, token)
            .await
        {
            Ok(id) => id,
            Err(_) => return Err(AuthenticationError::InvalidToken.into()),
        };

        if user_identifier != user_id {
            return Err(AuthenticationError::InvalidToken.into());
        }

        let user = self
            .database
            .update_user_password(&user_id, password)
            .await?;

        self.cache.delete_token(CacheId::PWToken, token).await?;

        self.generate_session_response(user).await
    }

    /// Generates a 200 OK HTTP response with a CSRF token and the user's session.
    pub(crate) async fn generate_session_response(
        &self,
        user: User,
    ) -> Result<HttpResponse, Error> {
        let session = self.database.create_session(&user).await?;

        let csrf_token = generate_hmac("CSRF_SECRET", Some(&session.id))?;

        let session_cookie = cookie::create("session", &session, None)?;

        let csrf_cookie = cookie::csrf(&csrf_token);

        // Delete login attempts on success
        match self.cache.delete_login_attempts(&user.id).await {
            Ok(_) => info!("Deleted cached login attempts for {}", user.id),
            Err(_) => info!("No login attempts found for user {}, proceeding", user.id),
        };

        // Cache the session initially
        self.cache.set_session(&csrf_token, &session).await?;

        Ok(AuthenticationSuccess::new(user, session).to_response(
            StatusCode::OK,
            Some(vec![session_cookie, csrf_cookie]),
            None,
        ))
    }
}
