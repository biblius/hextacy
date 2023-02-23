use crate::error::Error;
use actix_web::{cookie::Cookie, dev::ServiceRequest};
use storage::{models::role::Role, models::session::Session};

pub(crate) trait AuthGuardContract {
    fn get_valid_session(&self, session_id: &str, csrf: &str) -> Result<Session, Error>;
    fn get_csrf_header<'a>(&self, reg: &'a ServiceRequest) -> Result<&'a str, Error>;
    fn get_session_cookie(&self, reg: &ServiceRequest) -> Result<Cookie, Error>;
    fn check_valid_role(&self, role: &Role) -> bool;
    fn extract_user_session(&self, id: &str, csrf: &str) -> Result<Session, Error>;
}

pub(crate) trait CacheContract {
    fn get_session_by_id(&self, id: &str) -> Result<Session, Error>;
    fn cache_session(&self, csrf: &str, session: &Session) -> Result<(), Error>;
    fn refresh_session(&self, session_id: &str) -> Result<(), Error>;
}

pub(crate) trait RepoContract {
    fn refresh_session(&self, id: &str, csrf: &str) -> Result<Session, Error>;
    fn get_valid_session(&self, id: &str, csrf: &str) -> Result<Session, Error>;
}
