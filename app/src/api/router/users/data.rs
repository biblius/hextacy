use crate::models::user::SortOptions;
use serde::Deserialize;
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
