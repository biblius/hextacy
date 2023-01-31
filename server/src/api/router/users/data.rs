use alx_core::web::http::response::Response;
use derive_new::new;
use serde::{Deserialize, Serialize};
use storage::models::user::{SortOptions, User};
use validify::validify;

#[derive(Debug, Deserialize)]
#[validify]
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

impl Response<'_> for UserResponse {}
