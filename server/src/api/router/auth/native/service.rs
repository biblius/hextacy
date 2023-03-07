use super::{
    contract::ServiceContract,
    data::{
        ChangePassword, Credentials, EmailToken, ForgotPassword, ForgotPasswordVerify,
        FreezeAccountResponse, Logout, Otp, RegistrationData, RegistrationStartResponse,
        ResendRegToken, ResetPassword, TwoFactorAuthResponse,
    },
};
use crate::{
    api::router::auth::contract::{CacheContract, EmailContract},
    config::{
        cache::AuthCache,
        constants::{
            COOKIE_S_ID, MAXIMUM_LOGIN_ATTEMPTS, OTP_THROTTLE_INCREMENT, OTP_TOKEN_DURATION,
            REGISTRATION_TOKEN_DURATION, RESET_PW_TOKEN_DURATION, SESSION_DURATION,
        },
    },
};
use crate::{
    api::router::auth::{contract::RepositoryContract, data::AuthenticationSuccessResponse},
    error::{AuthenticationError, Error},
};
use actix_web::{body::BoxBody, HttpResponse, HttpResponseBuilder};
use alx_core::{
    crypto::{
        self,
        hmac::{generate_hmac, verify_hmac},
        {bcrypt_hash, bcrypt_verify, pw_and_hash, token, uuid},
    },
    web::http::{
        cookie,
        response::{MessageResponse, Response},
    },
};
use async_trait::async_trait;
use data_encoding::{BASE32, BASE64URL};
use reqwest::{
    header::{self, HeaderName, HeaderValue},
    StatusCode,
};
use storage::{
    adapters::AdapterError,
    models::{session::Session, user::User},
};
use tracing::{debug, info};

pub(super) struct Authentication<R, C, E> {
    pub repository: R,
    pub cache: C,
    pub email: E,
}

#[async_trait(?Send)]
impl<R, C, E> ServiceContract for Authentication<R, C, E>
where
    R: RepositoryContract,
    C: CacheContract,
    E: EmailContract,
{
    /// Verifies the user's credentials and returns a response based on their 2fa status
    async fn login(&self, credentials: Credentials) -> Result<HttpResponse, Error> {
        let Credentials {
            ref email,
            ref password,
            remember,
        } = credentials;

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

        if user.password.is_none() {
            return Err(AuthenticationError::InvalidCredentials.into());
        }

        // Check the password and cache the attempt if it was wrong
        if !bcrypt_verify(password, user.password.as_ref().unwrap())? {
            let attempts = self.cache.cache_login_attempt(&user.id)?;

            if attempts <= MAXIMUM_LOGIN_ATTEMPTS as u8 {
                return Err(AuthenticationError::InvalidCredentials.into());
            }

            // Freeze the account if attempts exceed the threshold and send a password reset token
            self.repository.freeze_user(&user.id).await?;

            let token = token(BASE64URL, 160);

            self.email
                .send_freeze_account(&user.username, &user.email, &token)?;

            self.cache.set_token(
                AuthCache::PWToken,
                &token,
                &user.id,
                Some(RESET_PW_TOKEN_DURATION),
            )?;

            return Ok(FreezeAccountResponse::new(
                &user.email,
                "Your account has been frozen due to too many invalid login attempts",
            )
            .to_response(StatusCode::LOCKED)
            .finish());
        }

        // If the user has 2FA turned on, stop here and cache the user ID so we can quickly verify their otp
        if user.otp_secret.is_some() {
            let token = token(BASE64URL, 80);

            debug!("User {} requires 2FA, caching token {}", user.id, token);

            self.cache.set_token(
                AuthCache::OTPToken,
                &token,
                &user.id,
                Some(OTP_TOKEN_DURATION),
            )?;

            return Ok(TwoFactorAuthResponse::new(&user.username, &token, remember)
                .to_response(StatusCode::OK)
                .finish());
        }
        self.establish_session(user, remember).await
    }

    /// Verifies the given OTP using the token generated on the credentials login. Throttles by 2*attempts seconds on each failed attempt.
    async fn verify_otp(&self, otp: Otp) -> Result<HttpResponse, Error> {
        let Otp {
            ref password,
            ref token,
            remember,
        } = otp;

        let user_id = match self.cache.get_token(AuthCache::OTPToken, token) {
            Ok(id) => id,
            Err(_) => return Err(AuthenticationError::InvalidToken("OTP").into()),
        };

        info!("Verifying OTP for {user_id}");

        let user = self.repository.get_user_by_id(&user_id).await?;

        let Some(ref secret) = user.otp_secret else {
            return Err(AuthenticationError::InvalidOTP.into());
        };

        // Check if there's an active throttle
        let attempts = self
            .cache
            .get_otp_throttle(AuthCache::OTPAttempts, &user.id)
            .ok();

        // Check whether it's ok to attempt to verify it
        if let Some(attempts) = attempts {
            let throttle = self
                .cache
                .get_otp_throttle(AuthCache::OTPThrottle, &user.id)?;
            let now = chrono::Utc::now().timestamp();
            if now - throttle <= OTP_THROTTLE_INCREMENT * attempts {
                return Err(AuthenticationError::AuthBlocked.into());
            }
        }
        let result = crypto::otp::verify_otp(password, secret)?;

        // If it's wrong increment the throttle and error
        if !result {
            self.cache.cache_otp_throttle(&user.id)?;
            return Err(AuthenticationError::InvalidOTP.into());
        }

        self.cache.delete_token(AuthCache::OTPToken, token)?;

        if attempts.is_some() {
            self.cache.delete_otp_throttle(&user.id)?;
        }

        self.establish_session(user, remember).await
    }

    /// Stores the initial data in the users table and sends an email to the user with the registration token.
    async fn start_registration(&self, data: RegistrationData) -> Result<HttpResponse, Error> {
        let RegistrationData {
            ref email,
            ref username,
            ref password,
        } = data;

        info!("Starting registration for {}", email);

        match self.repository.get_user_by_email(email).await {
            Ok(_) => return Err(AuthenticationError::EmailTaken.into()),
            Err(Error::Adapter(AdapterError::DoesNotExist)) => {}
            Err(e) => return Err(e.into()),
        }

        // TODO: handle existing oauth accounts on reg
        /* if let Ok(user) = self.repository.user.get_by_email(email) {

            // if user.password.is_none()
            return Err(AuthenticationError::EmailTaken.into());
        } */

        let hashed = bcrypt_hash(password)?;

        let user = self
            .repository
            .create_user(email, username, &hashed)
            .await?;

        let token = generate_hmac("REG_TOKEN_SECRET", &user.id, BASE64URL)?;

        self.cache.set_token(
            AuthCache::RegToken,
            &token,
            &user.id,
            Some(REGISTRATION_TOKEN_DURATION),
        )?;

        self.email
            .send_registration_token(&token, &user.username, email)?;

        Ok(RegistrationStartResponse::new(
            "Successfully sent registration token",
            &user.username,
            &user.email,
        )
        .to_response(StatusCode::CREATED)
        .finish())
    }

    /// Verifies the registration token sent via email upon registration.
    async fn verify_registration_token(&self, data: EmailToken) -> Result<HttpResponse, Error> {
        let token = &data.token;

        let user_id = match self.cache.get_token(AuthCache::RegToken, token) {
            Ok(id) => id,
            Err(_) => return Err(AuthenticationError::InvalidToken("Registration").into()),
        };

        info!("Verfiying registration token for {user_id}");

        // Verify the token with the hashed user ID, error if they mismatch
        if !verify_hmac("REG_TOKEN_SECRET", &user_id, token, BASE64URL)? {
            return Err(AuthenticationError::InvalidToken("Registration").into());
        }

        self.repository
            .update_user_email_verification(&user_id)
            .await?;

        self.cache.delete_token(AuthCache::RegToken, token)?;

        Ok(
            MessageResponse::new("Successfully verified registration token. Good job.")
                .to_response(StatusCode::OK)
                .finish(),
        )
    }

    /// Resends a registration token to the user if they are not already verified
    async fn resend_registration_token(&self, data: ResendRegToken) -> Result<HttpResponse, Error> {
        let email = data.email.as_str();
        info!("Resending registration token to {email}");

        // If the user does not exist we need to prevent erroring so as to not share
        // which emails exist in the db
        let response = MessageResponse::new(
            "An email will be sent with further instructions if it exists in our database.",
        )
        .to_response(StatusCode::OK)
        .finish();

        let user = match self.repository.get_user_by_email(email).await {
            Ok(u) => u,
            Err(Error::Adapter(AdapterError::DoesNotExist)) => return Ok(response),
            Err(e) => return Err(e),
        };

        if user.email_verified_at.is_some() {
            return Ok(response);
        }

        if self.cache.get_email_throttle(&user.id).ok().is_some() {
            return Err(AuthenticationError::AuthBlocked.into());
        }

        let token = token(BASE64URL, 160);

        self.cache.set_token(
            AuthCache::RegToken,
            &token,
            &user.id,
            Some(REGISTRATION_TOKEN_DURATION),
        )?;

        self.email
            .send_registration_token(&token, &user.username, &user.email)?;

        self.cache.set_email_throttle(&user.id)?;

        Ok(response)
    }

    /// Generates an OTP secret for the user and returns it in a QR code in the response. Requires a valid
    /// session beforehand.
    async fn set_otp_secret(&self, session: Session) -> Result<HttpResponse, Error> {
        let Session { ref user_id, .. } = session;

        let secret = crypto::otp::generate_secret();

        let user = self
            .repository
            .update_user_otp_secret(user_id, &secret)
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
        session: Session,
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

        self.cache.set_token(
            AuthCache::PWToken,
            &token,
            &user.id,
            Some(RESET_PW_TOKEN_DURATION),
        )?;

        self.email
            .alert_password_change(&user.username, &user.email, &token)?;

        info!("Successfully changed password for {}", session.user_id);

        Ok(MessageResponse::new("Successfully changed password. All sessions have been purged, please log in again to continue.")
        .to_response(StatusCode::OK)
        .finish())
    }

    /// Verify the forgot pw email token and update the user's password
    async fn verify_forgot_password(
        &self,
        data: ForgotPasswordVerify,
    ) -> Result<HttpResponse, Error> {
        info!("Verifying forgot password");

        let ForgotPasswordVerify {
            ref password,
            ref token,
        } = data;

        let user_id = match self.cache.get_token(AuthCache::PWToken, token) {
            Ok(id) => id,
            Err(_) => return Err(AuthenticationError::InvalidToken("Password").into()),
        };

        self.cache.delete_token(AuthCache::PWToken, token)?;

        let hashed = bcrypt_hash(password)?;

        let user = self
            .repository
            .update_user_password(&user_id, &hashed)
            .await?;

        self.purge_sessions(&user.id, None).await?;

        self.establish_session(user, false).await
    }

    /// Updates a user's password to a random string and sends it to them in the email
    async fn reset_password(&self, data: ResetPassword) -> Result<HttpResponse, Error> {
        let pw_token = data.token.as_str();

        // Check if there's a reset PW token in the cache
        let user_id = match self.cache.get_token(AuthCache::PWToken, pw_token) {
            Ok(id) => id,
            Err(_) => return Err(Error::new(AuthenticationError::InvalidToken("Password"))),
        };

        info!("Resetting password for {user_id}");

        self.cache.delete_token(AuthCache::PWToken, pw_token)?;

        // Create a temporary password
        let (temp_pw, hash) = pw_and_hash()?;
        let user = self
            .repository
            .update_user_password(&user_id, &hash)
            .await?;

        self.email
            .send_reset_password(&user.username, &user.email, &temp_pw)?;

        self.purge_sessions(&user.id, None).await?;

        Ok(
            MessageResponse::new("Successfully reset password. Incoming email.")
                .to_response(StatusCode::OK)
                .finish(),
        )
    }

    /// Resets the user's password and sends an email with a temporary one. Guarded by a half min throttle.
    async fn forgot_password(&self, data: ForgotPassword) -> Result<HttpResponse, Error> {
        info!("{} forgot password, sending email", data.email);

        let email = data.email.as_str();

        // If the user does not exist we need to prevent erroring so as to not share
        // which emails exist in the db
        let response = MessageResponse::new(
            "An email will be sent with further instructions if it exists in our database.",
        )
        .to_response(StatusCode::OK)
        .finish();

        let user = match self.repository.get_user_by_email(email).await {
            Ok(u) => u,
            Err(Error::Adapter(AdapterError::DoesNotExist)) => return Ok(response),
            Err(e) => return Err(e),
        };

        // Check throttle
        if self.cache.get_email_throttle(&user.id).ok().is_some() {
            return Err(AuthenticationError::AuthBlocked.into());
        }

        // Send and cache the temp password, throttle
        let token = token(BASE32, 20);

        self.email
            .send_forgot_password(&user.username, email, &token)?;

        self.cache.set_token(
            AuthCache::PWToken,
            &token,
            &user.id,
            Some(RESET_PW_TOKEN_DURATION),
        )?;

        self.cache.set_email_throttle(&user.id)?;

        Ok(response)
    }

    /// Deletes the user's current session and if purge is true expires all their sessions
    async fn logout(&self, session: Session, data: Logout) -> Result<HttpResponse, Error> {
        info!("Logging out {}", session.username);

        if data.purge {
            self.purge_sessions(&session.user_id, None).await?;
        } else {
            let session = self.repository.expire_session(&session.id).await?;
            self.cache.delete_token(AuthCache::Session, &session.id)?;
        }

        // Expire the cookie
        let cookie = cookie::create(COOKIE_S_ID, &session.id, true)?;

        Ok(MessageResponse::new("Successfully logged out, bye!")
            .to_response(StatusCode::OK)
            .with_cookies(vec![cookie])
            .finish())
    }

    /// Expires all sessions in the database and deletes all corresponding cached sessions
    async fn purge_sessions<'a>(&self, user_id: &str, skip: Option<&'a str>) -> Result<(), Error> {
        let sessions = self.repository.purge_sessions(user_id, skip).await?;
        for s in sessions {
            self.cache.delete_token(AuthCache::Session, &s.id).ok();
        }
        Ok(())
    }

    /// Generates a 200 OK HTTP response with a CSRF token in the headers and the user's session in a cookie.
    async fn establish_session(&self, user: User, remember: bool) -> Result<HttpResponse, Error> {
        let csrf_token = uuid();

        let session = self
            .repository
            .create_session(
                &user,
                &csrf_token,
                { !remember }.then_some(SESSION_DURATION),
                None,
                None,
            )
            .await?;

        let session_cookie = cookie::create(COOKIE_S_ID, &session.id, false)?;

        // Delete login attempts on success
        match self.cache.delete_login_attempts(&user.id) {
            Ok(_) => debug!("Deleted cached login attempts for {}", user.id),
            Err(_) => debug!("No login attempts found for user {}, proceeding", user.id),
        };

        // Cache the session
        self.cache.set_session(&session.id, &session)?;

        info!("Successfully created session for {}", user.username);

        // Respond with the x-csrf header and the session ID
        Ok(AuthenticationSuccessResponse::new(user)
            .to_response(StatusCode::OK)
            .with_cookies(vec![session_cookie])
            .with_headers(vec![(
                HeaderName::from_static("x-csrf-token"),
                HeaderValue::from_str(&csrf_token)?,
            )])
            .finish())
    }
}
