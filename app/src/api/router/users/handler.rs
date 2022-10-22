use super::{contract::ServiceContract, data::GetUsersPaginated};
use crate::error::Error;
use actix_web::{web, Responder};
use tracing::info;

pub(super) async fn get_paginated<S: ServiceContract>(
    data: web::Query<GetUsersPaginated>,
    service: web::Data<S>,
) -> Result<impl Responder, Error> {
    info!("Getting users");
    service.get_paginated(data.0).await
}
