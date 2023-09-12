use crate::cache::{contracts::BasicCacheAccess, TokenType};
use crate::config::constants::{
    EMAIL_THROTTLE_DURATION, OTP_THROTTLE_DURATION, OTP_TOKEN_DURATION,
    REGISTRATION_TOKEN_DURATION, RESET_PW_TOKEN_DURATION, SESSION_CACHE_DURATION,
    WRONG_PASSWORD_CACHE_DURATION,
};
use crate::db::models::session;
use crate::error::Error;
use chrono::Utc;
use hextacy::{component, contract};

#[component(
    use Driver as driver,
    use CacheAccess
)]
pub struct AuthenticationCacheAccess {}

#[component(
    use Driver for C,
    use BasicCacheAccess with C as Cache
)]
#[contract]
impl AuthenticationCacheAccess {
    /// Sessions get cached behind the user's csrf token.
    async fn set_session(&self, session_id: &str, session: &session::Session) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::set_json(
            &mut conn,
            TokenType::Session,
            session_id,
            session,
            Some(SESSION_CACHE_DURATION),
        )
        .await
        .map_err(Error::new)
    }

    async fn delete_session(&self, session_id: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::delete(&mut conn, TokenType::Session, session_id)
            .await
            .map_err(Error::new)
    }

    // Get

    async fn get_registration_token(&self, token: &str) -> Result<String, Error> {
        let mut conn = self.driver.connect().await?;
        Cache::get_string(&mut conn, TokenType::RegToken, token)
            .await
            .map_err(Error::new)
    }

    async fn get_pw_token(&self, token: &str) -> Result<String, Error> {
        let mut conn = self.driver.connect().await?;
        Cache::get_string(&mut conn, TokenType::PWToken, token)
            .await
            .map_err(Error::new)
    }

    async fn get_otp_token(&self, token: &str) -> Result<String, Error> {
        let mut conn = self.driver.connect().await?;
        Cache::get_string(&mut conn, TokenType::OTPToken, token)
            .await
            .map_err(Error::new)
    }

    async fn get_otp_throttle(&self, token: &str) -> Result<i64, Error> {
        let mut conn = self.driver.connect().await?;
        Cache::get_i64(&mut conn, TokenType::OTPThrottle, token)
            .await
            .map_err(Error::new)
    }

    async fn get_otp_attempts(&self, token: &str) -> Result<i64, Error> {
        let mut conn = self.driver.connect().await?;
        Cache::get_i64(&mut conn, TokenType::OTPAttempts, token)
            .await
            .map_err(Error::new)
    }

    // Delete

    async fn delete_registration_token(&self, token: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::delete(&mut conn, TokenType::RegToken, token)
            .await
            .map_err(Error::new)
    }

    async fn delete_pw_token(&self, token: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::delete(&mut conn, TokenType::PWToken, token)
            .await
            .map_err(Error::new)
    }

    async fn delete_otp_token(&self, token: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::delete(&mut conn, TokenType::OTPToken, token)
            .await
            .map_err(Error::new)
    }

    async fn delete_otp_attempts(&self, token: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::delete(&mut conn, TokenType::OTPAttempts, token)
            .await
            .map_err(Error::new)
    }

    // Set

    async fn set_registration_token(&self, token: &str, value: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::set_str(
            &mut conn,
            TokenType::RegToken,
            token,
            value,
            Some(REGISTRATION_TOKEN_DURATION),
        )
        .await
        .map_err(Error::new)
    }

    async fn set_pw_token(&self, token: &str, value: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::set_str(
            &mut conn,
            TokenType::PWToken,
            token,
            value,
            Some(RESET_PW_TOKEN_DURATION),
        )
        .await
        .map_err(Error::new)
    }

    async fn set_otp_token(&self, token: &str, value: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::set_str(
            &mut conn,
            TokenType::OTPToken,
            token,
            value,
            Some(OTP_TOKEN_DURATION),
        )
        .await
        .map_err(Error::new)
    }

    /// Caches the number of login attempts using the user ID as the key. If the attempts do not exist they
    /// will be created, otherwise they will be incremented.
    async fn cache_login_attempt(&self, user_id: &str) -> Result<i64, Error> {
        let mut conn = self.driver.connect().await?;
        Cache::set_or_increment(
            &mut conn,
            TokenType::LoginAttempts,
            user_id,
            1,
            Some(WRONG_PASSWORD_CACHE_DURATION),
        )
        .await
        .map_err(Error::new)
    }

    /// Removes the user's login attempts from the cache
    async fn delete_login_attempts(&self, user_id: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::delete(&mut conn, TokenType::LoginAttempts, user_id)
            .await
            .map_err(Error::new)
    }

    /// Cache the OTP throttle and attempts. The throttle always gets set to now and the attempts always get
    /// incremented. The domain should take care of the actual throttling.
    async fn cache_otp_throttle(&self, user_id: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;

        let attempts = Cache::get_i64(&mut conn, TokenType::OTPAttempts, user_id).await;

        match attempts {
            Ok(attempts) => {
                Cache::set_i64(
                    &mut conn,
                    TokenType::OTPThrottle,
                    user_id,
                    Utc::now().timestamp(),
                    Some(OTP_THROTTLE_DURATION),
                )
                .await?;

                Cache::set_i64(
                    &mut conn,
                    TokenType::OTPAttempts,
                    user_id,
                    attempts + 1,
                    Some(OTP_THROTTLE_DURATION),
                )
                .await?;
            }
            Err(_) => {
                Cache::set_i64(
                    &mut conn,
                    TokenType::OTPThrottle,
                    user_id,
                    Utc::now().timestamp(),
                    Some(OTP_THROTTLE_DURATION),
                )
                .await?;

                Cache::set_i64(
                    &mut conn,
                    TokenType::OTPAttempts,
                    user_id,
                    1,
                    Some(OTP_THROTTLE_DURATION),
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn set_email_throttle(&self, user_id: &str) -> Result<(), Error> {
        let mut conn = self.driver.connect().await?;
        Cache::set_i64(
            &mut conn,
            TokenType::EmailThrottle,
            user_id,
            1,
            Some(EMAIL_THROTTLE_DURATION),
        )
        .await
        .map_err(Error::new)
    }

    async fn get_email_throttle(&self, user_id: &str) -> Result<i64, Error> {
        let mut conn = self.driver.connect().await?;
        Cache::get_i64(&mut conn, TokenType::EmailThrottle, user_id)
            .await
            .map_err(Error::new)
    }
}
