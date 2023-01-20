/// Every session gets cached for a minute.
pub const SESSION_CACHE_DURATION_SECONDS: usize = 600;

/// Store the token for 2 days after a password change.
pub const RESET_PW_TOKEN_DURATION_SECONDS: usize = 172800;

/// Cache invalid login attempts for 2 days (172800 seconds). If the threshold is reached freeze the user's account.
pub const WRONG_PASSWORD_CACHE_DURATION: usize = 172800;

/// OTP EmailToken duration
pub const OTP_TOKEN_DURATION_SECONDS: usize = 300;

/// First time registration token duration
pub const REGISTRATION_TOKEN_DURATION_SECONDS: usize = 86400;

/// Maximum invalid logins until account freeze.
pub const MAXIMUM_LOGIN_ATTEMPTS: usize = 5;

/// OTP wrong attempt throttle duration.
pub const OTP_THROTTLE_DURATION_SECONDS: usize = 300;

/// OTP wrong attempt throttle increment.
pub const OTP_THROTTLE_INCREMENT: i64 = 3;

/// Throttle emails for half a minute to stop craziness
pub const EMAIL_THROTTLE_DURATION_SECONDS: usize = 30;

pub const EMAIL_DIRECTORY: &str = "resources/emails";

pub const COOKIE_S_ID: &str = "S_ID";
