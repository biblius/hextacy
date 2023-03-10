use crate::db::models::user::User;
use derive_new::new;
use hextacy::web::http::response::Response;
use serde::Serialize;

/// Sent when the user completely authenticates
#[derive(Debug, Serialize, new)]
pub(super) struct AuthenticationSuccessResponse {
    user: User,
}
impl Response<'_> for AuthenticationSuccessResponse {}
