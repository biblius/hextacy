use crate::{
    error::Error,
    models::user::{SortOptions, User},
};
use infrastructure::storage::postgres::Pg;
use std::sync::Arc;

pub(super) struct Postgres {
    pool: Arc<Pg>,
}

impl Postgres {
    pub(super) fn new(pool: Arc<Pg>) -> Self {
        Self { pool }
    }

    pub(super) fn get_paginated(
        &self,
        page: u16,
        per_page: u16,
        sort_by: Option<SortOptions>,
    ) -> Result<(i64, Vec<User>), Error> {
        User::get_paginated(
            page,
            per_page,
            sort_by.unwrap_or(SortOptions::CreatedAtDesc),
            &mut self.pool.connect().unwrap(),
        )
    }
}
