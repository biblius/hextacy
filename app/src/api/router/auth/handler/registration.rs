use super::super::data::registration::{EmailToken, RegistrationData};
use crate::{
    api::router::auth::{data::registration::SetPassword, service::authentication::Authentication},
    error::Error,
};
use actix_web::{web, Responder};
use tracing::info;

pub(crate) async fn start_registration(
    data: web::Form<RegistrationData>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Start Registration: {:?}", data.0);
    service.start_registration(data.0).await
}

pub(crate) async fn verify_registration_token(
    token: web::Query<EmailToken>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Verify registration token: {:?}", token);
    service.verify_registration_token(token.inner()).await
}

pub(crate) async fn set_password(
    user_id: web::Path<String>,
    data: web::Form<SetPassword>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Set password: {:?}", data);
    service.set_password(user_id.to_owned(), data.0).await
}
