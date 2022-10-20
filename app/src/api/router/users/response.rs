use infrastructure::http::response::Response;
use serde::Serialize;

use crate::models::user::User;

#[derive(Debug, Serialize)]
pub struct UserResponse {
    count: usize,
    users: Vec<User>,
}

impl UserResponse {
    pub fn new(count: usize, users: Vec<User>) -> Self {
        Self { count, users }
    }
}

impl Response for UserResponse {}
