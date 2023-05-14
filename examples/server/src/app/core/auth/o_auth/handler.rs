use super::{
    super::data::{OAuthCodeExchange, OAuthCodeExchangePayload},
    service::OAuthServiceContract,
};
use crate::{
    error::Error,
    helpers::request::extract_session,
    services::oauth::{github::GithubOAuth, google::GoogleOAuth, OAuthProvider},
};
use actix_web::{web, HttpRequest, Responder};
use tracing::info;
use validify::Validify;

pub async fn login<T: OAuthServiceContract>(
    path: web::Path<String>,
    data: web::Json<OAuthCodeExchangePayload>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    info!("Starting OAuth login");
    let code = OAuthCodeExchange::validify(data.0)?;
    let provider: OAuthProvider = path.to_string().try_into()?;
    match provider {
        OAuthProvider::Google => service.login(GoogleOAuth, code).await,
        OAuthProvider::Github => service.login(GithubOAuth, code).await,
    }
}

pub async fn request_scopes<T: OAuthServiceContract>(
    req: HttpRequest,
    path: web::Path<String>,
    data: web::Json<OAuthCodeExchangePayload>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    info!("OAuth requesting additional scopes");
    let code = OAuthCodeExchange::validify(data.0)?;
    let session = extract_session(req)?;
    let provider: OAuthProvider = path.to_string().try_into()?;
    match provider {
        OAuthProvider::Google => {
            service
                .request_additional_scopes(GoogleOAuth, session, code)
                .await
        }
        OAuthProvider::Github => {
            service
                .request_additional_scopes(GithubOAuth, session, code)
                .await
        }
    }
}
