use crate::{
    app::{
        core::auth::{
            data::{OAuthCodeExchange, OAuthCodeExchangePayload},
            o_auth::OAuthServiceContract,
        },
        router::AppResponse,
    },
    services::oauth::OAuthProviders,
};
use crate::{error::Error, helpers::request::extract_session, services::oauth::OAuthProvider};
use actix_web::{web, HttpRequest, Responder};
use tracing::info;
use validify::Validify;

pub async fn login<T: OAuthServiceContract>(
    providers: web::Data<OAuthProviders>,
    path: web::Path<String>,
    data: web::Json<OAuthCodeExchangePayload>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    info!("Starting OAuth login {:?}", path);
    let code = OAuthCodeExchange::validify(data.0)?;
    let provider: OAuthProvider = path.into_inner().try_into()?;

    match provider {
        OAuthProvider::Google => service
            .login(providers.google.clone(), code)
            .await
            .map(AppResponse),
        OAuthProvider::Github => service
            .login(providers.github.clone(), code)
            .await
            .map(AppResponse),
    }
}

pub async fn request_scopes<T: OAuthServiceContract>(
    providers: web::Data<OAuthProviders>,
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
        OAuthProvider::Google => service
            .request_additional_scopes(providers.google.clone(), session, code)
            .await
            .map(AppResponse),
        OAuthProvider::Github => service
            .request_additional_scopes(providers.github.clone(), session, code)
            .await
            .map(AppResponse),
    }
}
