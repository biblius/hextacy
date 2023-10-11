use super::MessageResponse;
use crate::config::state::AuthenticationService;
use crate::controllers::http::session_cookie;
use crate::core::models::session::Session;
use crate::error::Error;
use axum::extract::State;
use axum::http::{Response, StatusCode};
use axum::{Extension, Json};
use hextacy::web::xhttp::response::RestResponse;
use serde::Deserialize;
use validify::Validify;

#[derive(Debug, Deserialize, Validify)]
pub struct Register {
    #[modify(trim)]
    #[validate(length(min = 2))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validify)]
pub struct Login {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub remember: bool,
}

#[derive(Debug, Deserialize, Validify)]
pub struct Logout {
    pub purge: bool,
}

pub async fn register(
    State(service): State<AuthenticationService>,
    Json(data): Json<RegisterPayload>,
) -> Result<Response<String>, Error> {
    let Register { username, password } = Register::validify(data).map_err(Error::new)?;
    let session = service.register(&username, &password).await?;
    let (session_id, csrf) = (session.id.to_string(), session.csrf.to_string());
    let cookie = session_cookie("S_ID", &session_id, false);
    MessageResponse::new("Successfully created account")
        .into_response(StatusCode::CREATED)
        .with_headers([("x-csrf-token", &csrf)])
        .with_cookies(&[cookie])?
        .json()
        .map_err(Error::new)
}

pub async fn login(
    State(service): State<AuthenticationService>,
    Json(data): Json<LoginPayload>,
) -> Result<Response<String>, Error> {
    let Login {
        username,
        password,
        remember,
    } = Login::validify(data).map_err(Error::new)?;
    let session = service.login(&username, &password, remember).await?;
    let (session_id, csrf) = (session.id.to_string(), session.csrf.to_string());
    let cookie = session_cookie("S_ID", &session_id, false);
    MessageResponse::new("Successfully logged in")
        .into_response(StatusCode::OK)
        .with_headers([("x-csrf-token", &csrf)])
        .with_cookies(&[cookie])?
        .json()
        .map_err(Error::new)
}

pub async fn logout(
    State(service): State<AuthenticationService>,
    Extension(session): axum::extract::Extension<Session>,
    Json(data): Json<LogoutPayload>,
) -> Result<Response<String>, Error> {
    let Logout { purge } = Logout::validify(data).map_err(Error::new)?;
    let count = service.logout(session.id, purge).await?;
    let message = if count > 1 {
        format!("Successfully nuked {count} sessions")
    } else {
        "Successfully logged out".to_string()
    };
    MessageResponse::new(&message)
        .into_response(StatusCode::OK)
        .json()
        .map_err(Error::new)
}
