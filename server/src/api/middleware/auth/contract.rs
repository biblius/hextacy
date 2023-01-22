use crate::error::Error;
use actix_web::{cookie::Cookie, dev::ServiceRequest};
use storage::{models::role::Role, models::session::UserSession};

pub(crate) trait AuthGuardContract {
    fn get_valid_session(&self, session_id: &str, csrf: &str) -> Result<UserSession, Error>;
    fn get_csrf_header<'a>(&self, reg: &'a ServiceRequest) -> Result<&'a str, Error>;
    fn get_session_cookie(&self, reg: &ServiceRequest) -> Result<Cookie, Error>;
    fn check_valid_role(&self, role: &Role) -> bool;
    fn extract_user_session(&self, id: &str, csrf: &str) -> Result<UserSession, Error>;
}

pub(crate) trait CacheContract {
    fn get_session_by_id(&self, id: &str) -> Result<UserSession, Error>;
    fn cache_session(&self, csrf: &str, session: &UserSession) -> Result<(), Error>;
    fn refresh_session(&self, session_id: &str) -> Result<(), Error>;
}
