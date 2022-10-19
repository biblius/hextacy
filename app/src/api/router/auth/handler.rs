use crate::error::Error;
use actix_web::{web, Responder};
use tracing::info;

use super::{
    data::{Credentials, EmailToken, Otp, RegistrationData, SetPassword},
    service::authentication::Authentication,
};

pub(super) async fn verify_credentials(
    auth_form: web::Form<Credentials>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Credentials login : {:?}", auth_form.0);
    service.verify_credentials(auth_form.0).await
}

pub(super) async fn verify_otp(
    otp: web::Form<Otp>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("OTP login : {:?}", otp.0);
    service.verify_otp(otp.0).await
}

pub(super) async fn start_registration(
    data: web::Form<RegistrationData>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Start Registration: {:?}", data.0);
    service.start_registration(data.0).await
}

pub(super) async fn verify_registration_token(
    token: web::Query<EmailToken>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Verify registration token: {:?}", token);
    service.verify_registration_token(token.inner()).await
}

pub(super) async fn set_password(
    user_id: web::Path<String>,
    data: web::Form<SetPassword>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Set password: {:?}", data);
    service.set_password(user_id.as_str(), data.0).await
}

pub(super) async fn set_otp_secret(
    user_id: web::Path<String>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Registering OTP secret for: {}", user_id);
    service.set_otp_secret(user_id.as_str()).await
}
