use cookie::{time::Duration, SameSite};

/// Every session gets cached for a minute.
pub const SESSION_CACHE_DURATION_SECONDS: usize = 60;

/// Cache invalid login attempts for 2 days (172800 seconds). If the threshold is reached freeze the user's account.
pub const WRONG_PASSWORD_CACHE_DURATION: usize = 172800;

/// OTP Token duration
pub const OTP_TOKEN_DURATION_SECONDS: usize = 300;

/// First time registration password token duration
pub const PW_TOKEN_DURATION_SECONDS: usize = 300;

pub const REGISTRATION_TOKEN_EXPIRATION_SECONDS: usize = 300;

/// Maximum invalid logins until account freeze.
pub const MAXIMUM_LOGIN_ATTEMPTS: usize = 5;

pub const JWT_EXPIRATION: Duration = Duration::days(1);

pub const COOKIE_EXPIRATION: Duration = Duration::days(1);

pub const COOKIE_SAME_SITE_DEFAULT: SameSite = SameSite::Lax;
