use infrastructure::http::response::Response;
use serde::Serialize;

use crate::models::user::User;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    users: Vec<User>,
}

impl UserResponse {
    pub fn new(users: Vec<User>) -> Self {
        Self { users }
    }
}

impl Response for UserResponse {}
