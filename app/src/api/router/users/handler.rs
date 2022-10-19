use crate::{error::Error, models::session::Session};
use actix_web::{web, HttpMessage, HttpRequest, Responder};
use tracing::info;

use super::service::Users;

pub(crate) async fn get_all(
    service: web::Data<Users>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    info!("Request extension: {:?}", req.extensions().get::<Session>());
    info!("Getting all users");
    service.get_all().await
}
