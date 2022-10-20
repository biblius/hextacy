use super::{data::GetUsersPaginated, service::Users};
use crate::error::Error;
use actix_web::{web, Responder};
use tracing::info;

pub(super) async fn get_paginated(
    data: web::Query<GetUsersPaginated>,
    service: web::Data<Users>,
) -> Result<impl Responder, Error> {
    info!("Getting users");
    service.get_paginated(data.0).await
}
