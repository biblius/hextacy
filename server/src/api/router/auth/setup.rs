use super::{
    domain::Authentication,
    handler,
    infrastructure::{Cache, Email},
};
use crate::api::middleware::auth::interceptor;
use actix_web::web::{self, Data};
use alx_core::{
    clients::{postgres::Postgres, redis::Redis},
    services::email::lettre::SmtpTransport,
};
use std::sync::Arc;
use storage::adapters::postgres::{session::PgSessionAdapter, user::PgUserAdapter};
use storage::models::role::Role;

pub(crate) fn routes(
    pg: Arc<Postgres>,
    rd: Arc<Redis>,
    email: Arc<SmtpTransport>,
    cfg: &mut web::ServiceConfig,
) {
    let service = Authentication {
        user_repo: PgUserAdapter { client: pg.clone() },
        session_repo: PgSessionAdapter { client: pg.clone() },
        cache: Cache { client: rd.clone() },
        email: Email { client: email },
    };
    let auth_guard = interceptor::AuthGuard::new(pg, rd, Role::User);
    cfg.app_data(Data::new(service));

    cfg.service(
        web::resource("/auth/login").route(
            web::post().to(handler::login::<
                Authentication<PgUserAdapter, PgSessionAdapter, Cache, Email>,
            >),
        ),
    );
    cfg.service(web::resource("/auth/register").route(
        web::post().to(handler::start_registration::<
            Authentication<PgUserAdapter, PgSessionAdapter, Cache, Email>,
        >),
    ));
    cfg.service(
        web::resource("/auth/verify-registration-token").route(web::get().to(
            handler::verify_registration_token::<
                Authentication<PgUserAdapter, PgSessionAdapter, Cache, Email>,
            >,
        )),
    );
    cfg.service(
        web::resource("/auth/resend-registration-token").route(web::post().to(
            handler::resend_registration_token::<
                Authentication<PgUserAdapter, PgSessionAdapter, Cache, Email>,
            >,
        )),
    );
    cfg.service(
        web::resource("/auth/set-otp")
            .route(web::get().to(handler::set_otp_secret::<
                Authentication<PgUserAdapter, PgSessionAdapter, Cache, Email>,
            >))
            .wrap(auth_guard.clone()),
    );
    cfg.service(web::resource("/auth/verify-otp").route(
        web::post().to(handler::verify_otp::<
            Authentication<PgUserAdapter, PgSessionAdapter, Cache, Email>,
        >),
    ));
    cfg.service(
        web::resource("/auth/change-password")
            .route(web::post().to(handler::change_password::<
                Authentication<PgUserAdapter, PgSessionAdapter, Cache, Email>,
            >))
            .wrap(auth_guard.clone()),
    );
    cfg.service(web::resource("/auth/forgot-password").route(web::post().to(
        handler::forgot_password::<Authentication<PgUserAdapter, PgSessionAdapter, Cache, Email>>,
    )));
    cfg.service(
        web::resource("/auth/verify-forgot-password").route(web::post().to(
            handler::verify_forgot_password::<
                Authentication<PgUserAdapter, PgSessionAdapter, Cache, Email>,
            >,
        )),
    );
    cfg.service(web::resource("/auth/reset-password").route(web::get().to(
        handler::reset_password::<Authentication<PgUserAdapter, PgSessionAdapter, Cache, Email>>,
    )));
    cfg.service(
        web::resource("/auth/logout")
            .route(web::post().to(handler::logout::<
                Authentication<PgUserAdapter, PgSessionAdapter, Cache, Email>,
            >))
            .wrap(auth_guard),
    );
}
