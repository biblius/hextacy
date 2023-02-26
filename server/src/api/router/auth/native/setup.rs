use super::{handler, service::Authentication};
use crate::api::middleware::auth::interceptor;
use crate::api::router::auth::adapter::{Cache, Email, Repository};
use actix_web::web::{self, Data};
use alx_core::clients::db::postgres::PgPoolConnection;
use alx_core::clients::{db::postgres::Postgres, db::redis::Redis, email::Email as EmailClient};
use std::sync::Arc;
use storage::adapters::postgres::oauth::PgOAuthAdapter;
use storage::adapters::postgres::{session::PgSessionAdapter, user::PgUserAdapter};
use storage::models::role::Role;

pub(crate) fn routes(
    pg: Arc<Postgres>,
    rd: Arc<Redis>,
    email: Arc<EmailClient>,
    cfg: &mut web::ServiceConfig,
) {
    let service = Authentication {
        repo: Repository::<
            Postgres,
            PgPoolConnection,
            PgUserAdapter,
            PgSessionAdapter,
            PgOAuthAdapter,
        >::new(pg.clone()),
        cache: Cache { client: rd.clone() },
        email: Email { client: email },
    };
    let auth_guard = interceptor::AuthGuard::new(pg, rd, Role::User);

    cfg.app_data(Data::new(service));

    cfg.service(
        web::resource("/auth/login").route(web::post().to(handler::login::<
            Authentication<
                Repository<
                    Postgres,
                    PgPoolConnection,
                    PgUserAdapter,
                    PgSessionAdapter,
                    PgOAuthAdapter,
                >,
                Cache,
                Email,
            >,
        >)),
    );

    cfg.service(web::resource("/auth/register").route(web::post().to(
        handler::start_registration::<
            Authentication<
                Repository<
                    Postgres,
                    PgPoolConnection,
                    PgUserAdapter,
                    PgSessionAdapter,
                    PgOAuthAdapter,
                >,
                Cache,
                Email,
            >,
        >,
    )));

    cfg.service(
        web::resource("/auth/verify-registration-token").route(web::get().to(
            handler::verify_registration_token::<
                Authentication<
                    Repository<
                        Postgres,
                        PgPoolConnection,
                        PgUserAdapter,
                        PgSessionAdapter,
                        PgOAuthAdapter,
                    >,
                    Cache,
                    Email,
                >,
            >,
        )),
    );

    cfg.service(
        web::resource("/auth/resend-registration-token").route(web::post().to(
            handler::resend_registration_token::<
                Authentication<
                    Repository<
                        Postgres,
                        PgPoolConnection,
                        PgUserAdapter,
                        PgSessionAdapter,
                        PgOAuthAdapter,
                    >,
                    Cache,
                    Email,
                >,
            >,
        )),
    );

    cfg.service(
        web::resource("/auth/set-otp")
            .route(web::get().to(handler::set_otp_secret::<
                Authentication<
                    Repository<
                        Postgres,
                        PgPoolConnection,
                        PgUserAdapter,
                        PgSessionAdapter,
                        PgOAuthAdapter,
                    >,
                    Cache,
                    Email,
                >,
            >))
            .wrap(auth_guard.clone()),
    );

    cfg.service(
        web::resource("/auth/verify-otp").route(web::post().to(handler::verify_otp::<
            Authentication<
                Repository<
                    Postgres,
                    PgPoolConnection,
                    PgUserAdapter,
                    PgSessionAdapter,
                    PgOAuthAdapter,
                >,
                Cache,
                Email,
            >,
        >)),
    );

    cfg.service(
        web::resource("/auth/change-password")
            .route(web::post().to(handler::change_password::<
                Authentication<
                    Repository<
                        Postgres,
                        PgPoolConnection,
                        PgUserAdapter,
                        PgSessionAdapter,
                        PgOAuthAdapter,
                    >,
                    Cache,
                    Email,
                >,
            >))
            .wrap(auth_guard.clone()),
    );

    cfg.service(web::resource("/auth/forgot-password").route(web::post().to(
        handler::forgot_password::<
            Authentication<
                Repository<
                    Postgres,
                    PgPoolConnection,
                    PgUserAdapter,
                    PgSessionAdapter,
                    PgOAuthAdapter,
                >,
                Cache,
                Email,
            >,
        >,
    )));

    cfg.service(
        web::resource("/auth/verify-forgot-password").route(web::post().to(
            handler::verify_forgot_password::<
                Authentication<
                    Repository<
                        Postgres,
                        PgPoolConnection,
                        PgUserAdapter,
                        PgSessionAdapter,
                        PgOAuthAdapter,
                    >,
                    Cache,
                    Email,
                >,
            >,
        )),
    );

    cfg.service(web::resource("/auth/reset-password").route(web::get().to(
        handler::reset_password::<
            Authentication<
                Repository<
                    Postgres,
                    PgPoolConnection,
                    PgUserAdapter,
                    PgSessionAdapter,
                    PgOAuthAdapter,
                >,
                Cache,
                Email,
            >,
        >,
    )));

    cfg.service(
        web::resource("/auth/logout")
            .route(web::post().to(handler::logout::<
                Authentication<
                    Repository<
                        Postgres,
                        PgPoolConnection,
                        PgUserAdapter,
                        PgSessionAdapter,
                        PgOAuthAdapter,
                    >,
                    Cache,
                    Email,
                >,
            >))
            .wrap(auth_guard),
    );
}
