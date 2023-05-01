use super::{handler, service::Authentication};
use crate::api::middleware::auth::{
    adapter::{Cache as MwCache, Repo as MwRepo},
    interceptor,
};
use crate::api::router::auth::adapters::{cache::Cache, email::Email, repository::Repo};
use crate::db::models::role::Role;
use actix_web::web::{self, Data};
use hextacy::drivers::{cache::redis::Redis, db::postgres::seaorm::PostgresSea};
use hextacy::drivers::{db::postgres::diesel::PostgresDiesel, email::Email as EmailClient};
use hextacy::{drivers::db::mongo::Mongo, route};
use std::sync::Arc;

pub(crate) fn routes(
    _pg: Arc<PostgresDiesel>,
    pg: Arc<PostgresSea>,
    rd: Arc<Redis>,
    email: Arc<EmailClient>,
    _mg: Arc<Mongo>,
    cfg: &mut web::ServiceConfig,
) {
    let service = Authentication {
        repository: Repo::new(pg),
        cache: Cache { driver: rd.clone() },
        email: Email { driver: email },
    };

    let session_guard =
        interceptor::AuthenticationGuard::<MwRepo, MwCache>::new(_pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    route!(
        Authentication<Repo, Cache, Email>, cfg,

        post => "/auth/login" => login;

        post => "/auth/register" => start_registration;

        get => "/auth/verify-registration-token" => verify_registration_token;

        post => "/auth/resend-registration-token" => resend_registration_token;

        get => "/auth/set-otp" => set_otp_secret;

        post => "/auth/verify-otp" => verify_otp;

        post => "/auth/change-password" => change_password | session_guard;

        post => "/auth/forgot-password" => forgot_password;

        post => "/auth/verify-forgot-password" => verify_forgot_password;

        get => "/auth/reset-password" => reset_password;

        post => "/auth/logout" => logout | session_guard;
    );
}
