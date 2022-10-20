use std::sync::Arc;

use super::infrastructure::{cache::Cache, email::Email, postgres::Postgres};
use crate::{
    api::router::auth::{
        data::{Credentials, Otp, RegistrationData, SetPassword},
        response::{
            AuthenticationSuccess, FreezeAccount, Prompt2FA, RegistrationSuccess, ResendPWToken,
            TokenVerified,
        },
    },
    error::{AuthenticationError, Error},
    models::user::User,
};
use actix_web::{body::BoxBody, cookie::SameSite, HttpResponse, HttpResponseBuilder};
use data_encoding::BASE64URL;
use infrastructure::{
    config::constants::{
        MAXIMUM_LOGIN_ATTEMPTS, OTP_TOKEN_DURATION_SECONDS, PW_TOKEN_DURATION_SECONDS,
        REGISTRATION_TOKEN_EXPIRATION_SECONDS,
    },
    crypto::{
        self,
        utility::{bcrypt_verify, generate_hmac, uuid},
    },
    email::lettre::SmtpTransport,
    http::{cookie, response::Response},
    storage::{
        postgres::Pg,
        redis::{CacheId, Rd},
    },
};
use reqwest::{
    header::{self, HeaderName, HeaderValue},
    StatusCode,
};
use tracing::info;

pub(super) struct Authentication {
    database: Postgres,
    cache: Cache,
    email: Email,
}

// #[async_trait]
impl Authentication {
    pub(super) fn new(pg: Arc<Pg>, rd: Arc<Rd>, email: Arc<SmtpTransport>) -> Self {
        Self {
            database: Postgres::new(pg),
            cache: Cache::new(rd),
            email: Email::new(email),
        }
    }

    /// Verifies the user's credentials and returns a response based on their 2fa status
    pub(super) async fn verify_credentials(
        &self,
        credentials: Credentials,
    ) -> Result<HttpResponse, Error> {
        let (email, password) = credentials.data();

        info!("Verifying credentials for {}", email);

        let user = match self.database.get_user_by_email(email).await {
            Ok(u) => u,
            Err(_) => return Err(AuthenticationError::InvalidCredentials.into()),
        };

        // If the account is frozen
        if user.frozen {
            return Err(AuthenticationError::AccountFrozen.into());
        }

        // Unverified email
        if user.email_verified_at.is_none() {
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        // If the user doesn't have their pw set up, resend a temporary password token
        if user.password.is_none() {
            let token = crypto::utility::token(BASE64URL)?;

            self.cache
                .set_token(
                    CacheId::PWToken,
                    &token,
                    &user.id,
                    Some(PW_TOKEN_DURATION_SECONDS),
                )
                .await?;

            return Ok(
                ResendPWToken::new("Please set your password to continue", &token).to_response(
                    StatusCode::OK,
                    None,
                    None,
                ),
            );
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
            info!(
                "verfiy_credentials - User {:?} requires 2FA, caching token",
                user.id
            );

            let token = generate_hmac("OTP_HMAC_SECRET", &user.password.unwrap(), BASE64URL)?;

            self.cache
                .set_token(
                    CacheId::OTPToken,
                    &token,
                    &user.id,
                    Some(OTP_TOKEN_DURATION_SECONDS),
                )
                .await?;

            return Ok(Prompt2FA::new(&user.username, &token).to_response(
                StatusCode::OK,
                None,
                None,
            ));
        }

        self.generate_session_response(user).await
    }

    /// Verifies the given OTP using the token generated on the credentials login.
    pub(super) async fn verify_otp(&self, otp: Otp) -> Result<HttpResponse, Error> {
        let (password, token) = otp.data();

        let user_id = match self
            .cache
            .get_token::<String>(CacheId::OTPToken, token)
            .await
        {
            Ok(id) => id,
            Err(_) => return Err(AuthenticationError::InvalidToken(CacheId::OTPToken).into()),
        };

        info!("Verifying OTP for {} ", user_id);

        let user = self.database.get_user_by_id(&user_id).await?;

        // Something went wrong if the user doesn't have a pw set up here
        if user.password.is_none() {
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        // Verify the user's token that was created from their password
        if !crypto::utility::verify_hmac(
            "OTP_HMAC_SECRET",
            user.password.as_ref().unwrap(),
            token,
            BASE64URL,
        )? {
            return Err(AuthenticationError::InvalidToken(CacheId::OTPToken).into());
        }

        if let Some(ref secret) = user.otp_secret {
            let (result, _) = crypto::utility::verify_otp(password, secret)?;

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
    pub(super) async fn start_registration(
        &self,
        data: RegistrationData,
    ) -> Result<HttpResponse, Error> {
        let (email, username) = data.inner();

        info!("Starting registration for {}", email);

        if self.database.get_user_by_email(email).await.is_ok() {
            return Err(AuthenticationError::EmailTaken.into());
        }

        let user = self.database.create_user(email, username).await?;

        let token = generate_hmac("EMAIL_HMAC_SECRET", &user.id, BASE64URL)?;

        self.cache
            .set_token(
                CacheId::RegToken,
                &token,
                &user.id,
                Some(REGISTRATION_TOKEN_EXPIRATION_SECONDS),
            )
            .await?;

        self.email
            .send_registration_token(&token, &user.username, email)?;

        Ok(RegistrationSuccess::new(
            "Successfully sent registration token",
            &user.username,
            &user.email,
        )
        .to_response(StatusCode::CREATED, None, None))
    }

    /// Verifies the registration token sent via email upon registration. Upon success, generates
    /// a one time password token to be used when setting a password.
    pub(super) async fn verify_registration_token(
        &self,
        token: &str,
    ) -> Result<HttpResponse, Error> {
        let user_id = match self
            .cache
            .get_token::<String>(CacheId::RegToken, token)
            .await
        {
            Ok(id) => id,
            Err(_) => return Err(AuthenticationError::InvalidToken(CacheId::RegToken).into()),
        };

        info!("Verfiying registration token for {user_id}");

        // Verify the token with the hashed user ID, error if they mismatch
        if !crypto::utility::verify_hmac("EMAIL_HMAC_SECRET", &user_id, token, BASE64URL)? {
            return Err(AuthenticationError::InvalidToken(CacheId::RegToken).into());
        }

        self.database.update_email_verified_at(&user_id).await?;

        self.cache.delete_token(CacheId::RegToken, token).await?;

        let pw_token = generate_hmac("PW_TOKEN_SECRET", &user_id, BASE64URL)?;

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

    /// Set the user's password after successful email token verification
    pub(super) async fn set_password(&self, data: SetPassword) -> Result<HttpResponse, Error> {
        let (token, password) = data.inner();

        let user_id = match self
            .cache
            .get_token::<String>(CacheId::PWToken, token)
            .await
        {
            Ok(id) => id,
            Err(_) => return Err(AuthenticationError::InvalidToken(CacheId::PWToken).into()),
        };

        let user = self
            .database
            .update_user_password(&user_id, password)
            .await?;

        self.cache.delete_token(CacheId::PWToken, token).await?;

        info!("Successfully set password for {user_id}");

        self.generate_session_response(user).await
    }

    /// Generates an OTP secret for the user and returns it in a QR code in the response
    pub(super) async fn set_otp_secret(&self, user_id: &str) -> Result<HttpResponse, Error> {
        let secret = crypto::utility::generate_otp_secret();

        let user = self.database.set_user_otp_secret(user_id, &secret).await?;

        let qr = crypto::utility::generate_totp_qr_code(&secret, &user.email)?;

        info!("Successfully set OTP secret for {}", user.id);

        let response = HttpResponseBuilder::new(StatusCode::OK)
            .append_header((
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("image/svg+xml"),
            ))
            .body(BoxBody::new(qr));

        Ok(response)
    }

    /// Generates a 200 OK HTTP response with a CSRF token in the headers and the user's session in a cookie.
    async fn generate_session_response(&self, user: User) -> Result<HttpResponse, Error> {
        let csrf_token = uuid();

        let session = self.database.create_session(&user, &csrf_token).await?;

        let session_cookie = cookie::create("session_id", &session.id, Some(SameSite::None))?;

        // Delete login attempts on success
        match self.cache.delete_login_attempts(&user.id).await {
            Ok(_) => info!("Deleted cached login attempts for {}", user.id),
            Err(_) => info!("No login attempts found for user {}, proceeding", user.id),
        };

        // Cache the session initially
        self.cache.set_session(&csrf_token, &session).await?;

        info!("Successfully created session for {}", user.id);

        // Respond with the x-csrf header and the session ID
        Ok(AuthenticationSuccess::new(user, session).to_response(
            StatusCode::OK,
            Some(vec![session_cookie]),
            Some(vec![(
                HeaderName::from_static("x-csrf-token"),
                HeaderValue::from_str(&csrf_token)?,
            )]),
        ))
    }
}
