use derive_new::new;
use infrastructure::store::repository::user::{SortOptions, User};
use infrastructure::web::http::response::Response;
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

#[derive(Debug, Serialize, new)]
pub struct UserResponse {
    users: Vec<User>,
}

impl Response for UserResponse {}
