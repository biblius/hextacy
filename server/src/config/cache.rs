use hextacy::cache::CacheIdentifier;

#[derive(Debug, PartialEq, Eq)]
pub enum AuthCache {
    /// For keeping track of login attempts
    LoginAttempts,
    /// For caching sessions
    Session,
    /// For keeping track of registration tokens
    RegToken,
    /// For keeping track of password reset tokens
    PWToken,
    /// For 2FA login, OTPs won't be accepted without this token in the cache
    OTPToken,
    /// For 2FA login failure
    OTPThrottle,
    /// For 2FA login failure
    OTPAttempts,
    /// For stopping email craziness
    EmailThrottle,
}

impl CacheIdentifier for AuthCache {
    fn id(self) -> &'static str {
        match self {
            AuthCache::LoginAttempts => "login_attempts",
            AuthCache::Session => "session",
            AuthCache::RegToken => "registration_token",
            AuthCache::PWToken => "set_pw",
            AuthCache::OTPToken => "otp",
            AuthCache::OTPThrottle => "otp_throttle",
            AuthCache::OTPAttempts => "otp_attempts",
            AuthCache::EmailThrottle => "emal_throttle",
        }
    }
}
