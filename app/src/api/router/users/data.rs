use infrastructure::repository::user::{SortOptions, User};
use infrastructure::utility::http::response::Response;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub(super) struct GetUsersPaginated {
    #[validate(range(min = 1, max = 65_535))]
    pub page: Option<u16>,
    #[validate(range(min = 1, max = 65_535))]
    pub per_page: Option<u16>,
    pub sort_by: Option<SortOptions>,
}

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
