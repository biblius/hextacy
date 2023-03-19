//! All duration values are in seconds

/// Valid session duration in seconds, one day.
pub const SESSION_DURATION: i64 = 60 * 60 * 24;

/// Session cache duration in seconds, 10 minutes.
pub const SESSION_CACHE_DURATION: usize = 60 * 10;

/// Store the token for 2 days after a password change.
pub const RESET_PW_TOKEN_DURATION: usize = 60 * 60 * 24 * 2;

/// Cache invalid login attempts for 2 days (172800 seconds). If the threshold is reached freeze the user's account.
pub const WRONG_PASSWORD_CACHE_DURATION: usize = 60 * 60 * 24 * 2;

/// OTP EmailToken duration
pub const OTP_TOKEN_DURATION: usize = 60 * 5;

/// First time registration token duration
pub const REGISTRATION_TOKEN_DURATION: usize = 60 * 60 * 24;

/// Maximum invalid logins until account freeze.
pub const MAXIMUM_LOGIN_ATTEMPTS: usize = 5;

/// OTP wrong attempt throttle duration.
pub const OTP_THROTTLE_DURATION: usize = 300;

/// OTP wrong attempt throttle increment. Increments by 3 seconds every time.
pub const OTP_THROTTLE_INCREMENT: i64 = 3;

/// Throttle emails for half a minute to stop craziness
pub const EMAIL_THROTTLE_DURATION: usize = 30;

pub const OPEN_SSL_KEY_PATH: &str = "openssl/key.pem";

pub const OPEN_SSL_CERT_PATH: &str = "openssl/cert.pem";

pub const EMAIL_DIRECTORY: &str = "resources/emails";

pub const FAVICON_PATH: &str = "resources/favicon.ico";

pub const COOKIE_S_ID: &str = "S_ID";
