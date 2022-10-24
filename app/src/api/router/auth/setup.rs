use super::{
    domain::Authentication,
    handler,
    infrastructure::{Cache, Email, Repository},
};
use crate::api::middleware::auth::interceptor;
use actix_web::web::{self, Data};
use infrastructure::{
    adapters::postgres::{session::PgSessionAdapter, user::PgUserAdapter},
    clients::{email::lettre::SmtpTransport, postgres::Postgres, redis::Redis},
    repository::role::Role,
};
use std::sync::Arc;

pub(in super::super) fn routes(
    pg: Arc<Postgres>,
    rd: Arc<Redis>,
    email: Arc<SmtpTransport>,
    cfg: &mut web::ServiceConfig,
) {
    let repository = Repository {
        user_repo: PgUserAdapter { client: pg.clone() },
        session_repo: PgSessionAdapter { client: pg.clone() },
    };
    let cache = Cache { client: rd.clone() };
    let email = Email {
        client: email.clone(),
    };
    let service = Authentication {
        repository,
        cache,
        email,
    };
    let guard = interceptor::AuthGuard::new(pg.clone(), rd.clone(), Role::User);

    cfg.app_data(Data::new(service));

    // Initial registration
    cfg.service(web::resource("/auth/register").route(web::post().to(
        handler::start_registration::<
            Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
        >,
    )));

    // Credentials login
    cfg.service(
        web::resource("/auth/login").route(web::post().to(handler::login::<
            Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
        >)),
    );

    // Logout
    cfg.service(
        web::resource("/auth/logout")
            .route(web::post().to(handler::logout::<
                Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
            >))
            .wrap(guard.clone()),
    );

    // OTP login
    cfg.service(
        web::resource("/auth/verify-otp").route(web::post().to(handler::verify_otp::<
            Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
        >)),
    );

    // Verify registration token
    cfg.service(
        web::resource("/auth/verify-registration-token").route(web::get().to(
            handler::verify_registration_token::<
                Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
            >,
        )),
    );

    // Change password
    cfg.service(
        web::resource("/auth/change-password")
            .route(web::post().to(handler::change_password::<
                Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
            >))
            .wrap(guard.clone()),
    );

    // Forgot password
    cfg.service(web::resource("/auth/forgot-password").route(web::post().to(
        handler::forgot_password::<
            Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
        >,
    )));

    // Reset password
    cfg.service(web::resource("/auth/reset-password").route(web::get().to(
        handler::reset_password::<
            Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
        >,
    )));

    // Set otp
    cfg.service(
        web::resource("/auth/set-otp")
            .route(web::get().to(handler::set_otp_secret::<
                Authentication<Repository<PgUserAdapter, PgSessionAdapter>, Cache, Email>,
            >))
            .wrap(guard),
    );
}
