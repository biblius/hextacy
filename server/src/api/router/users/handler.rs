use super::{contract::ServiceContract, data::GetUsersPaginated};
use crate::error::Error;
use actix_web::{web, Responder};
use tracing::info;
use validator::Validate;

pub(super) async fn get_paginated<T: ServiceContract>(
    data: web::Query<GetUsersPaginated>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;
    info!("Getting users");
    service.get_paginated(data.0).await
}
