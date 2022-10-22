use super::{
    contract::{CacheContract, EmailContract, RepositoryContract, ServiceContract},
    data::{
        AuthenticationSuccessResponse, ChangePassword, ChangePasswordResponse, Credentials,
        EmailToken, FreezeAccountResponse, Logout, LogoutResponse, Otp, RegistrationData,
        RegistrationSuccessResponse, TokenVerifiedResponse, TwoFactorAuthResponse,
    },
};
use crate::{
    error::{AuthenticationError, Error},
    services::cache::CacheId,
};
use actix_web::{body::BoxBody, HttpResponse, HttpResponseBuilder};
use async_trait::async_trait;
use data_encoding::BASE64URL;
use infrastructure::{
    config::constants::{
        MAXIMUM_LOGIN_ATTEMPTS, OTP_TOKEN_DURATION_SECONDS, REGISTRATION_TOKEN_DURATION_SECONDS,
        RESET_PW_TOKEN_DURATION_SECONDS,
    },
    crypto::{
        self,
        token::{generate_hmac, verify_hmac},
        utility::{bcrypt_hash, bcrypt_verify, token},
    },
    repository::user::User,
    utility::http::{cookie, response::Response},
};
use reqwest::{
    header::{self, HeaderName, HeaderValue},
    StatusCode,
};
use tracing::info;

pub(super) struct Authentication<R, C, E>
where
    R: RepositoryContract,
    C: CacheContract,
    E: EmailContract,
{
    pub repository: R,
    pub cache: C,
    pub email: E,
}

#[async_trait]
impl<R, C, E> ServiceContract for Authentication<R, C, E>
where
    R: RepositoryContract + Send + Sync,
    C: CacheContract + Send + Sync,
    E: EmailContract + Send + Sync,
{
    /// Verifies the user's credentials and returns a response based on their 2fa status
    async fn verify_credentials(&self, credentials: Credentials) -> Result<HttpResponse, Error> {
        let (email, password, remember) = (
            credentials.email.as_str(),
            credentials.password.as_str(),
            credentials.remember,
        );
        info!("Verifying credentials for {email}");

        let user = match self.repository.get_user_by_email(email).await {
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

        // Cache the attempt if it was wrong
        if !bcrypt_verify(password, user.password.as_str())? {
            let attempts = self.cache.cache_login_attempt(&user.id).await?;

            // Freeze the account if attempts exceed the threshold
            if attempts > MAXIMUM_LOGIN_ATTEMPTS as u8 {
                self.repository.freeze_user(&user.id).await?;
                return Ok(FreezeAccountResponse::new(
                    &user.id,
                    "Your account has been frozen due to too many invalid login attempts",
                )
                .to_response(StatusCode::LOCKED, None, None));
            }
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        // If the user has 2FA turned on, stop here and cache the user so we can quickly verify their otp
        if user.otp_secret.is_some() {
            info!("User {} requires 2FA, caching token", user.id);

            let token = generate_hmac("OTP_TOKEN_SECRET", &user.password, BASE64URL)?;

            self.cache
                .set_token(
                    CacheId::OTPToken,
                    &token,
                    &user.id,
                    Some(OTP_TOKEN_DURATION_SECONDS),
                )
                .await?;

            return Ok(
                TwoFactorAuthResponse::new(&user.username, &token, remember).to_response(
                    StatusCode::OK,
                    None,
                    None,
                ),
            );
        }

        self.session_response(user, remember).await
    }

    /// Verifies the given OTP using the token generated on the credentials login.
    async fn verify_otp(&self, otp: Otp) -> Result<HttpResponse, Error> {
        let (password, token, remember) = (otp.password.as_str(), otp.token.as_str(), otp.remember);

        let user_id = match self
            .cache
            .get_token::<String>(CacheId::OTPToken, token)
            .await
        {
            Ok(id) => id,
            Err(_) => return Err(AuthenticationError::InvalidToken(CacheId::OTPToken).into()),
        };

        info!("Verifying OTP for {} ", user_id);

        let user = self.repository.get_user_by_id(&user_id).await?;

        // Verify the user's token that was created from their password
        if !verify_hmac("OTP_TOKEN_SECRET", user.password.as_str(), token, BASE64URL)? {
            return Err(AuthenticationError::InvalidToken(CacheId::OTPToken).into());
        }

        if let Some(ref secret) = user.otp_secret {
            let (result, _) = crypto::otp::verify_otp(password, secret)?;

            if !result {
                return Err(AuthenticationError::InvalidOTP.into());
            }

            self.cache.delete_token(CacheId::OTPToken, token).await?;

            self.session_response(user, remember).await
        } else {
            Err(AuthenticationError::InvalidOTP.into())
        }
    }

    /// Stores the initial data in the users table and sends an email to the user with the registration token.
    async fn start_registration(&self, data: RegistrationData) -> Result<HttpResponse, Error> {
        let (email, username, password) = (
            data.email.as_str(),
            data.username.as_str(),
            data.password.as_str(),
        );

        info!("Starting registration for {}", email);

        if self.repository.get_user_by_email(email).await.is_ok() {
            return Err(AuthenticationError::EmailTaken.into());
        }
        let hashed = bcrypt_hash(password)?;
        let user = self
            .repository
            .create_user(email, username, &hashed)
            .await?;
        let token = generate_hmac("REG_TOKEN_SECRET", &user.id, BASE64URL)?;
        self.cache
            .set_token(
                CacheId::RegToken,
                &token,
                &user.id,
                Some(REGISTRATION_TOKEN_DURATION_SECONDS),
            )
            .await?;
        self.email
            .send_registration_token(&token, &user.username, email)
            .await?;

        Ok(RegistrationSuccessResponse::new(
            "Successfully sent registration token",
            &user.username,
            &user.email,
        )
        .to_response(StatusCode::CREATED, None, None))
    }

    /// Verifies the registration token sent via email upon registration.
    async fn verify_registration_token(&self, data: EmailToken) -> Result<HttpResponse, Error> {
        let token = &data.token;
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
        if !verify_hmac("REG_TOKEN_SECRET", &user_id, token, BASE64URL)? {
            return Err(AuthenticationError::InvalidToken(CacheId::RegToken).into());
        }
        self.repository.update_email_verified_at(&user_id).await?;
        self.cache.delete_token(CacheId::RegToken, token).await?;

        Ok(
            TokenVerifiedResponse::new(&user_id, "Successfully verified registration token")
                .to_response(StatusCode::OK, None, None),
        )
    }

    /// Generates an OTP secret for the user and returns it in a QR code in the response. Requires a valid
    /// session beforehand.
    async fn set_otp_secret(&self, user_id: &str) -> Result<HttpResponse, Error> {
        let secret = crypto::otp::generate_secret();
        let user = self
            .repository
            .set_user_otp_secret(user_id, &secret)
            .await?;
        let qr = crypto::otp::generate_totp_qr_code(&secret, &user.email)?;

        info!("Successfully set OTP secret for {}", user.id);

        Ok(HttpResponseBuilder::new(StatusCode::OK)
            .append_header((
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("image/svg+xml"),
            ))
            .body(BoxBody::new(qr)))
    }

    /// Update user password. Purge all sessions and notify by email.
    async fn change_password(
        &self,
        user_id: &str,
        data: ChangePassword,
    ) -> Result<HttpResponse, Error> {
        let password = data.password.as_str();
        let hashed = crypto::utility::bcrypt_hash(password)?;
        let user = self
            .repository
            .update_user_password(user_id, &hashed)
            .await?;

        self.purge_and_clear_sessions(&user.id).await?;
        let token = token(BASE64URL, 128)?;
        self.cache
            .set_token(
                CacheId::PWToken,
                &token,
                &user.id,
                Some(RESET_PW_TOKEN_DURATION_SECONDS),
            )
            .await?;
        self.email
            .alert_password_change(&token, &user.username, &user.email)
            .await?;

        info!("Successfully changed password for {user_id}");

        Ok(ChangePasswordResponse::new("Successfully changed password. All sessions have been purged, please log in again to continue.").to_response(StatusCode::OK, None, None))
    }

    /// Deletes the user's current session and if purge is true expires all their sessions
    async fn logout(
        &self,
        session_id: &str,
        user_id: &str,
        data: Logout,
    ) -> Result<HttpResponse, Error> {
        if data.purge {
            self.purge_and_clear_sessions(user_id).await?;
        } else {
            let session = self.repository.expire_session(session_id).await?;
            self.cache
                .delete_token(CacheId::Session, &session.csrf_token)
                .await?;
        }
        let cookie = cookie::create_session(session_id, true, false);

        Ok(
            LogoutResponse::new("Successfully logged out, bye!").to_response(
                StatusCode::OK,
                Some(vec![cookie]),
                None,
            ),
        )
    }

    /// Expires all sessions in the database and deletes all corresponding cached sessions
    async fn purge_and_clear_sessions(&self, user_id: &str) -> Result<(), Error> {
        let sessions = self.repository.purge_sessions(user_id).await?;
        for s in sessions {
            self.cache
                .delete_token(CacheId::Session, &s.csrf_token)
                .await
                .ok();
        }
        Ok(())
    }

    /// Generates a 200 OK HTTP response with a CSRF token in the headers and the user's session in a cookie.
    async fn session_response(&self, user: User, remember: bool) -> Result<HttpResponse, Error> {
        let csrf_token = token(BASE64URL, 160)?;
        let session = self
            .repository
            .create_session(&user, &csrf_token, remember)
            .await?;
        let session_cookie = cookie::create_session(&session.id, false, remember);

        // Delete login attempts on success
        match self.cache.delete_login_attempts(&user.id).await {
            Ok(_) => info!("Deleted cached login attempts for {}", user.id),
            Err(_) => info!("No login attempts found for user {}, proceeding", user.id),
        };

        // If the session is permanent, cache it initially
        if remember {
            self.cache.set_session(&csrf_token, &session).await?;
        }

        info!("Successfully created session for {}", user.id);

        // Respond with the x-csrf header and the session ID
        Ok(
            AuthenticationSuccessResponse::new(user, session.clone()).to_response(
                StatusCode::OK,
                Some(vec![session_cookie]),
                Some(vec![(
                    HeaderName::from_static("x-csrf-token"),
                    HeaderValue::from_str(&csrf_token)?,
                )]),
            ),
        )
    }
}