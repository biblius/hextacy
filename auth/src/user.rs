use serde::{Deserialize, Serialize};
use storage::diesel::Queryable;

#[derive(Debug, Deserialize, Serialize, Queryable)]
pub struct User {
    id: String,
    email: String,
    #[serde(skip_serializing)]
    password: String,
    first_name: String,
    last_name: String,
    #[serde(skip_serializing)]
    mfa_type: Option<String>,
    #[serde(skip_serializing)]
    otp_secret: Option<String>,
}

pub struct UserSession {
    csrf: String,
    user: User,
}
