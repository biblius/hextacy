use super::{
    handler::{
        change_password, forgot_password, login, logout, resend_registration_token, reset_password,
        set_otp_secret, start_registration, verify_forgot_password, verify_otp,
        verify_registration_token,
    },
    service::Authentication,
};
use crate::{
    api::middleware::auth::{
        adapter::{Cache as MwCache, Repo as MwRepo},
        interceptor,
    },
    api::router::auth::adapters::{
        cache::AuthenticationCache, email::Email, repository::RepositoryComponent,
    },
    config::AppState,
    db::adapters::postgres::seaorm::{
        oauth::PgOAuthAdapter, session::PgSessionAdapter, user::PgUserAdapter,
    },
};
use crate::{cache::adapters::redis::RedisAdapter, db::models::role::Role};
use actix_web::web::{self, Data};
use hextacy::{
    drivers::{
        cache::redis::{Redis, RedisConnection},
        db::postgres::seaorm::PostgresSea,
    },
    route,
};
use sea_orm::DatabaseConnection;

type CacheComponent = AuthenticationCache<Redis, RedisConnection, RedisAdapter>;

type RepoComponent = RepositoryComponent<
    PostgresSea,
    DatabaseConnection,
    PgUserAdapter,
    PgSessionAdapter,
    PgOAuthAdapter,
>;

pub(crate) fn routes(
    AppState {
        pg_sea,
        redis,
        smtp,
        ..
    }: &AppState,
    cfg: &mut web::ServiceConfig,
) {
    let service = Authentication {
        repository: RepoComponent::new(pg_sea.clone()),
        cache: CacheComponent::new(redis.clone()),
        email: Email {
            driver: smtp.clone(),
        },
    };

    let session_guard = interceptor::AuthenticationGuard::<MwRepo, MwCache>::new(
        pg_sea.clone(),
        redis.clone(),
        Role::User,
    );

    cfg.app_data(Data::new(service));

    route!(
        Authentication<RepoComponent, CacheComponent, Email>, cfg,

        post => "/auth/login" => login;

        post => "/auth/register" =>  start_registration;

        get => "/auth/verify-registration-token" => verify_registration_token;

        post => "/auth/resend-registration-token" => resend_registration_token;

        get => "/auth/set-otp" => set_otp_secret;

        post => "/auth/verify-otp" => verify_otp;

        post => "/auth/change-password" => |session_guard => change_password;

        post => "/auth/forgot-password" => forgot_password;

        post => "/auth/verify-forgot-password" => verify_forgot_password;

        get => "/auth/reset-password" => reset_password;

        post => "/auth/logout" => | session_guard => logout;
    );
}
