use super::{contract::ServiceContract, data::GetUsersPaginated};
use crate::error::Error;
use actix_web::{web, Responder};
use tracing::info;
use validify::Validify;

pub(super) async fn get_paginated<T: ServiceContract>(
    data: web::Query<<GetUsersPaginated as Validify>::Payload>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    let query = GetUsersPaginated::validify(data.0)?;
    info!("Getting users");
    service.get_paginated(query).await
}
