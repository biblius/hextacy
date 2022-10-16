use super::{
    super::data::login::{Credentials, Otp},
    super::service::authentication::Authentication,
};
use crate::error::Error;
use actix_web::{web, Responder};
use tracing::info;

pub(crate) async fn credentials(
    auth_form: web::Form<Credentials>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("Credentials login : {:?}", auth_form.0);
    service.verify_credentials(auth_form.0).await
}

pub(crate) async fn otp(
    otp: web::Form<Otp>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("OTP login : {:?}", otp.0);
    service.verify_otp(otp.0).await
}
