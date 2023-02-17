use super::{contract::ServiceContract, data::OAuthCodeExchangePayload};
use crate::{
    api::router::auth::o_auth::data::OAuthCodeExchange, error::Error,
    helpers::request::extract_session,
};
use actix_web::{web, HttpRequest, Responder};
use tracing::info;
use validify::Validify;

/// Verifies the user's login credentials and either establishes a session if the user
/// doesn't have 2FA or prompts the user for their 2FA pass if they have it set up
pub(super) async fn login<T: ServiceContract>(
    data: web::Json<OAuthCodeExchangePayload>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    let code = OAuthCodeExchange::validify(data.0)?;
    info!("OAuth login : {:?}", code);
    service.login(code).await
}

/// Verifies the user's login credentials and either establishes a session if the user
/// doesn't have 2FA or prompts the user for their 2FA pass if they have it set up
pub(super) async fn request_scopes<T: ServiceContract>(
    req: HttpRequest,
    data: web::Json<OAuthCodeExchangePayload>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    let code = OAuthCodeExchange::validify(data.0)?;
    let session = extract_session(req)?;
    info!("OAuth requesting additional scopes : {:?}", code);
    service.request_additional_scopes(session, code).await
}
