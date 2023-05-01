use crate::db::{models::role::Role, models::session::Session};
use crate::error::Error;
use actix_web::{cookie::Cookie, dev::ServiceRequest};
use async_trait::async_trait;

#[async_trait]
pub(crate) trait AuthGuardApi {
    async fn get_valid_session(&self, session_id: &str, csrf: &str) -> Result<Session, Error>;
    fn check_valid_role(&self, role: &Role) -> bool;
    fn get_csrf_header<'a>(&self, reg: &'a ServiceRequest) -> Result<&'a str, Error>;
    fn get_session_cookie(&self, reg: &ServiceRequest) -> Result<Cookie, Error>;
}

pub(crate) trait CacheApi {
    fn get_session_by_id(&self, id: &str) -> Result<Session, Error>;
    fn cache_session(&self, csrf: &str, session: &Session) -> Result<(), Error>;
    fn refresh_session(&self, session_id: &str) -> Result<(), Error>;
}

#[async_trait]
pub(crate) trait RepositoryApi {
    async fn refresh_session(&self, id: &str, csrf: &str) -> Result<Session, Error>;
    async fn get_valid_session(&self, id: &str, csrf: &str) -> Result<Session, Error>;
}
