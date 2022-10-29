use super::{
    contract::{CacheContract, EmailContract, RepositoryContract, ServiceContract},
    data::{
        AuthenticationSuccessResponse, ChangePassword, Credentials, EmailToken, ForgotPassword,
        ForgotPasswordVerify, FreezeAccountResponse, Logout, Otp, RegistrationData,
        RegistrationStartResponse, ResetPassword, TwoFactorAuthResponse,
    },
};
use crate::{
    error::{AuthenticationError, Error},
    helpers::cache::CacheId,
};
use actix_web::{body::BoxBody, HttpResponse, HttpResponseBuilder};
use async_trait::async_trait;
use data_encoding::{BASE32, BASE64URL};
use infrastructure::{
    config::constants::{
        EMAIL_THROTTLE_DURATION_SECONDS, MAXIMUM_LOGIN_ATTEMPTS, OTP_THROTTLE_INCREMENT,
        OTP_TOKEN_DURATION_SECONDS, REGISTRATION_TOKEN_DURATION_SECONDS,
        RESET_PW_TOKEN_DURATION_SECONDS,
    },
    crypto::{
        self,
        hmac::{generate_hmac, verify_hmac},
        utility::{bcrypt_hash, bcrypt_verify, pw_and_hash, token, uuid},
    },
    store::{models::user_session::UserSession, repository::user::User},
    web::http::{
        cookie,
        response::{MessageResponse, Response},
    },
};
use reqwest::{
    header::{self, HeaderName, HeaderValue},
    StatusCode,
};
use tracing::{debug, info};

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
    async fn login(&self, credentials: Credentials) -> Result<HttpResponse, Error> {
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
        if user.frozen {
            return Err(AuthenticationError::AccountFrozen.into());
        }
        if user.email_verified_at.is_none() {
            return Err(AuthenticationError::EmailUnverified.into());
        }
        // Check the password and cache the attempt if it was wrong
        if !bcrypt_verify(password, user.password.as_str())? {
            let attempts = self.cache.cache_login_attempt(&user.id).await?;
            // Freeze the account if attempts exceed the threshold and send a password reset token
            if attempts > MAXIMUM_LOGIN_ATTEMPTS as u8 {
                self.repository.freeze_user(&user.id).await?;
                let token = token(BASE64URL, 160);
                self.email
                    .send_freeze_account(&user.username, &user.email, &token)
                    .await?;
                self.cache
                    .set_token(
                        CacheId::PWToken,
                        &token,
                        &user.id,
                        Some(RESET_PW_TOKEN_DURATION_SECONDS),
                    )
                    .await?;
                return Ok(FreezeAccountResponse::new(
                    &user.email,
                    "Your account has been frozen due to too many invalid login attempts",
                )
                .to_response(StatusCode::LOCKED, None, None));
            }
            return Err(AuthenticationError::InvalidCredentials.into());
        }
        // If the user has 2FA turned on, stop here and cache the user ID so we can quickly verify their otp
        if user.otp_secret.is_some() {
            let token = token(BASE64URL, 80);
            debug!("User {} requires 2FA, caching token {}", user.id, token);
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

    /// Verifies the given OTP using the token generated on the credentials login. Throttles by 2*attempts seconds on each failed attempt.
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
        info!("Verifying OTP for {user_id}");
        let user = self.repository.get_user_by_id(&user_id).await?;
        if let Some(ref secret) = user.otp_secret {
            // Check if there's an active throttle
            let attempts = self
                .cache
                .get_token::<i64>(CacheId::OTPAttempts, &user.id)
                .await
                .ok();
            // Check whether it's ok to attempt to verify it
            if let Some(attempts) = attempts {
                let throttle = self
                    .cache
                    .get_token::<i64>(CacheId::OTPThrottle, &user.id)
                    .await?;
                let now = chrono::Utc::now().timestamp();
                if now - throttle <= OTP_THROTTLE_INCREMENT * attempts {
                    return Err(AuthenticationError::AuthBlocked.into());
                }
            }
            let (result, _) = crypto::otp::verify_otp(password, secret)?;
            // If it's wrong increment the throttle and error
            if !result {
                self.cache.cache_otp_throttle(&user.id).await?;
                return Err(AuthenticationError::InvalidOTP.into());
            }
            self.cache.delete_token(CacheId::OTPToken, token).await?;
            if attempts.is_some() {
                self.cache.delete_otp_throttle(&user.id).await?;
            }
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
        Ok(RegistrationStartResponse::new(
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
            MessageResponse::new("Successfully verified registration token. Good job.")
                .to_response(StatusCode::OK, None, None),
        )
    }

    /// Resends a registration token to the user if they are not already verified
    async fn resend_registration_token(&self, email: &str) -> Result<HttpResponse, Error> {
        info!("Resending registration token to {email}");
        let user = self.repository.get_user_by_email(email).await?;
        if user.email_verified_at.is_some() {
            return Err(Error::new(AuthenticationError::AlreadyVerified));
        }
        if self
            .cache
            .get_token::<i64>(CacheId::EmailThrottle, &user.id)
            .await
            .ok()
            .is_some()
        {
            return Err(AuthenticationError::AuthBlocked.into());
        }
        let token = token(BASE64URL, 160);
        self.cache
            .set_token(
                CacheId::RegToken,
                &token,
                &user.id,
                Some(REGISTRATION_TOKEN_DURATION_SECONDS),
            )
            .await?;
        self.email
            .send_registration_token(&token, &user.username, &user.email)
            .await?;
        self.cache
            .set_token(
                CacheId::OTPThrottle,
                &user.id,
                &1,
                Some(EMAIL_THROTTLE_DURATION_SECONDS),
            )
            .await?;
        Ok(
            MessageResponse::new("Successfully sent registration token. Incoming email.")
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

    /// Updates the user's password. Purges all sessions and sends an email with a reset token.
    async fn change_password(
        &self,
        session: UserSession,
        data: ChangePassword,
    ) -> Result<HttpResponse, Error> {
        let password = data.password.as_str();
        let hashed = bcrypt_hash(password)?;
        let user = self
            .repository
            .update_user_password(&session.user_id, &hashed)
            .await?;
        self.purge_sessions(&user.id, None).await?;
        let token = token(BASE64URL, 128);
        self.cache
            .set_token(
                CacheId::PWToken,
                &token,
                &user.id,
                Some(RESET_PW_TOKEN_DURATION_SECONDS),
            )
            .await?;
        self.email
            .alert_password_change(&user.username, &user.email, &token)
            .await?;

        info!("Successfully changed password for {}", session.user_id);
        Ok(MessageResponse::new("Successfully changed password. All sessions have been purged, please log in again to continue.").to_response(StatusCode::OK, None, None))
    }

    /// Updates a user's password to a random string and sends it to them in the email
    async fn reset_password(&self, data: ResetPassword) -> Result<HttpResponse, Error> {
        let pw_token = data.token.as_str();
        // Check if there's a reset PW token in the cache
        let user_id = match self
            .cache
            .get_token::<String>(CacheId::PWToken, pw_token)
            .await
        {
            Ok(id) => id,
            Err(_) => {
                return Err(Error::new(AuthenticationError::InvalidToken(
                    CacheId::PWToken,
                )))
            }
        };
        info!("Resetting password for {user_id}");
        self.cache.delete_token(CacheId::PWToken, pw_token).await?;
        // Create a temporary password
        let (temp_pw, hash) = pw_and_hash()?;
        let user = self
            .repository
            .update_user_password(&user_id, &hash)
            .await?;
        self.email
            .send_reset_password(&user.username, &user.email, &temp_pw)
            .await?;
        self.purge_sessions(&user.id, None).await?;
        Ok(
            MessageResponse::new("Successfully reset password. Incoming email.").to_response(
                StatusCode::OK,
                None,
                None,
            ),
        )
    }

    /// Resets the user's password and sends an email with a temporary one. Guarded by a half min throttle.
    async fn forgot_password(&self, data: ForgotPassword) -> Result<HttpResponse, Error> {
        info!("{} forgot password, sending email", data.email);
        let email = data.email.as_str();
        let user = self.repository.get_user_by_email(email).await?;
        // Check throttle
        if self
            .cache
            .get_token::<i64>(CacheId::EmailThrottle, &user.id)
            .await
            .ok()
            .is_some()
        {
            return Err(AuthenticationError::AuthBlocked.into());
        }
        // Send and cache the temp password, throttle
        let token = token(BASE32, 20);
        self.email
            .send_forgot_password(&user.username, email, &token)
            .await?;
        self.cache
            .set_token(
                CacheId::PWToken,
                &token,
                &user.id,
                Some(RESET_PW_TOKEN_DURATION_SECONDS),
            )
            .await?;
        self.cache
            .set_token(
                CacheId::EmailThrottle,
                &user.id,
                &1,
                Some(EMAIL_THROTTLE_DURATION_SECONDS),
            )
            .await?;
        Ok(
            MessageResponse::new("Started forgot password routine. Incoming email.").to_response(
                StatusCode::OK,
                None,
                None,
            ),
        )
    }

    /// Verify the forgot pw email token and update the user's password
    async fn verify_forgot_password(
        &self,
        data: ForgotPasswordVerify,
    ) -> Result<HttpResponse, Error> {
        info!("Verifying forgot password");
        let (password, token) = (data.password.as_str(), data.token.as_str());
        let user_id = match self
            .cache
            .get_token::<String>(CacheId::PWToken, token)
            .await
        {
            Ok(id) => id,
            Err(_) => return Err(AuthenticationError::InvalidToken(CacheId::PWToken).into()),
        };
        self.cache.delete_token(CacheId::PWToken, token).await?;
        let hashed = bcrypt_hash(password)?;
        let user = self
            .repository
            .update_user_password(&user_id, &hashed)
            .await?;

        self.purge_sessions(&user.id, None).await?;

        self.session_response(user, false).await
    }

    /// Deletes the user's current session and if purge is true expires all their sessions
    async fn logout(&self, session: UserSession, data: Logout) -> Result<HttpResponse, Error> {
        info!("Logging out {}", session.user_name);
        if data.purge {
            self.purge_sessions(&session.user_id, None).await?;
        } else {
            let session = self.repository.expire_session(&session.id).await?;
            self.cache
                .delete_token(CacheId::Session, &session.id)
                .await?;
        }
        // Expire the cookie
        let cookie = cookie::create_session(&session.id, true, false);
        Ok(
            MessageResponse::new("Successfully logged out, bye!").to_response(
                StatusCode::OK,
                Some(vec![cookie]),
                None,
            ),
        )
    }

    /// Expires all sessions in the database and deletes all corresponding cached sessions
    async fn purge_sessions<'a>(&self, user_id: &str, skip: Option<&'a str>) -> Result<(), Error> {
        let sessions = self.repository.purge_sessions(user_id, skip).await?;
        for s in sessions {
            self.cache.delete_token(CacheId::Session, &s.id).await.ok();
        }
        Ok(())
    }

    /// Generates a 200 OK HTTP response with a CSRF token in the headers and the user's session in a cookie.
    async fn session_response(&self, user: User, remember: bool) -> Result<HttpResponse, Error> {
        let csrf_token = uuid();
        let session = self
            .repository
            .create_session(&user, &csrf_token, remember)
            .await?;
        let session_cookie = cookie::create_session(&session.id, false, remember);
        // Delete login attempts on success
        match self.cache.delete_login_attempts(&user.id).await {
            Ok(_) => debug!("Deleted cached login attempts for {}", user.id),
            Err(_) => debug!("No login attempts found for user {}, proceeding", user.id),
        };
        // Cache the session
        self.cache
            .set_session(
                &session.id,
                &UserSession::new(user.clone(), session.clone()),
            )
            .await?;
        info!("Successfully created session for {}", user.username);
        // Respond with the x-csrf header and the session ID
        Ok(AuthenticationSuccessResponse::new(user).to_response(
            StatusCode::OK,
            Some(vec![session_cookie]),
            Some(vec![(
                HeaderName::from_static("x-csrf-token"),
                HeaderValue::from_str(&csrf_token)?,
            )]),
        ))
    }
}
