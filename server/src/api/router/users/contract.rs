use super::data::GetUsersPaginated;
use crate::error::Error;
use actix_web::HttpResponse;

pub(super) trait ServiceContract {
    fn get_paginated(&self, data: GetUsersPaginated) -> Result<HttpResponse, Error>;
}
