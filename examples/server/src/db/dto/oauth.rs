use super::super::adapters::postgres::diesel::schema::oauth;
use crate::services::oauth::OAuthProvider;
use diesel::{query_builder::AsChangeset, Insertable};
use hextacy::Constructor;
use std::fmt::Debug;

#[derive(Debug, Insertable, Constructor, AsChangeset)]
#[diesel(table_name = oauth)]
pub struct OAuthMetaData<'a> {
    pub user_id: &'a str,
    pub access_token: &'a str,
    pub refresh_token: Option<&'a str>,
    pub provider: OAuthProvider,
    pub account_id: Option<&'a str>,
    pub scope: &'a str,
}
