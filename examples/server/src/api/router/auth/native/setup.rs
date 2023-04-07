use super::{handler, service::Authentication};
use crate::api::middleware::auth::{
    adapter::{Cache as MwCache, Repo as MwRepo},
    interceptor,
};
use crate::api::router::auth::adapters::{cache::Cache, email::Email, repository::Repo};
use crate::db::models::role::Role;
use actix_web::web::{self, Data};
use hextacy::drivers::db::mongo::Mongo;
use hextacy::drivers::{cache::redis::Redis, db::postgres::seaorm::PostgresSea};
use hextacy::drivers::{db::postgres::diesel::PostgresDiesel, email::Email as EmailClient};
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
        repository: Repo::new(pg.clone()),
        cache: Cache { driver: rd.clone() },
        email: Email { driver: email },
    };
    let auth_guard = interceptor::AuthGuard::<MwRepo, MwCache>::new(_pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    cfg.service(
        web::resource("/auth/login")
            .route(web::post().to(handler::login::<Authentication<Repo, Cache, Email>>)),
    );

    cfg.service(
        web::resource("/auth/register").route(
            web::post().to(handler::start_registration::<Authentication<Repo, Cache, Email>>),
        ),
    );

    cfg.service(web::resource("/auth/verify-registration-token").route(
        web::get().to(handler::verify_registration_token::<Authentication<Repo, Cache, Email>>),
    ));

    cfg.service(web::resource("/auth/resend-registration-token").route(
        web::post().to(handler::resend_registration_token::<Authentication<Repo, Cache, Email>>),
    ));

    cfg.service(
        web::resource("/auth/set-otp")
            .route(web::get().to(handler::set_otp_secret::<Authentication<Repo, Cache, Email>>))
            .wrap(auth_guard.clone()),
    );

    cfg.service(
        web::resource("/auth/verify-otp")
            .route(web::post().to(handler::verify_otp::<Authentication<Repo, Cache, Email>>)),
    );

    cfg.service(
        web::resource("/auth/change-password")
            .route(web::post().to(handler::change_password::<Authentication<Repo, Cache, Email>>))
            .wrap(auth_guard.clone()),
    );

    cfg.service(
        web::resource("/auth/forgot-password")
            .route(web::post().to(handler::forgot_password::<Authentication<Repo, Cache, Email>>)),
    );

    cfg.service(web::resource("/auth/verify-forgot-password").route(
        web::post().to(handler::verify_forgot_password::<Authentication<Repo, Cache, Email>>),
    ));

    cfg.service(
        web::resource("/auth/reset-password")
            .route(web::get().to(handler::reset_password::<Authentication<Repo, Cache, Email>>)),
    );

    cfg.service(
        web::resource("/auth/logout")
            .route(web::post().to(handler::logout::<Authentication<Repo, Cache, Email>>))
            .wrap(auth_guard),
    );
}
