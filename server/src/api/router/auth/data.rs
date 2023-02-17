use alx_core::web::http::response::Response;
use derive_new::new;
use serde::Serialize;
use storage::models::user::User;

/// Sent when the user completely authenticates
#[derive(Debug, Serialize, new)]
pub(super) struct AuthenticationSuccessResponse {
    user: User,
}
impl Response<'_> for AuthenticationSuccessResponse {}
