use super::{data::Credentials, service::Authentication};
use crate::error::Error;
use actix_web::{web, Responder};
use tracing::info;

pub async fn credentials(
    auth_form: web::Form<Credentials<'_>>,
    service: web::Data<Authentication>,
) -> Result<impl Responder, Error> {
    info!("{}{:?}", "User login : ", auth_form);
    service.verify_credentials(auth_form.0).await
}
