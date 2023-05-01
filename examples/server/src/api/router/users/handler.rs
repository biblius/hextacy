use super::api::ServiceApi;
use crate::{
    api::router::users::data::{GetUsersPaginated, GetUsersPaginatedPayload},
    error::Error,
};
use actix_web::{web, Responder};
use tracing::info;
use validify::Validify;

pub struct Handler;

impl Handler {
    pub(super) async fn get_paginated<T: ServiceApi>(
        data: web::Query<GetUsersPaginatedPayload>,
        service: web::Data<T>,
    ) -> Result<impl Responder, Error> {
        let query = GetUsersPaginated::validify(data.0)?;
        info!("Getting users");
        service.get_paginated(query).await
    }
}
