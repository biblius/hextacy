use super::{
    contract::ServiceContract,
    data::{
        ChangePassword, Credentials, EmailToken, ForgotPassword, ForgotPasswordVerify, Logout, Otp,
        RegistrationData, ResendRegToken, ResetPassword,
    },
};
use crate::error::Error;
use actix_web::{web, HttpRequest, Responder};
use infrastructure::web::http::request::extract_session;
use tracing::info;
use validator::Validate;

/// Verifies the user's login credentials and either establishes a session if the user
/// doesn't have 2FA or prompts the user for their 2FA pass if they have it set up
pub(super) async fn login<T: ServiceContract>(
    data: web::Json<Credentials>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;
    info!("Credentials login : {:?}", data.0);
    service.login(data.0).await
}

/// Starts the registration process for the user and sends an email containing a temporary
/// token used to complete the registration
pub(super) async fn start_registration<T: ServiceContract>(
    data: web::Json<RegistrationData>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;
    info!("Start Registration: {:?}", data.0);
    service.start_registration(data.0).await
}

/// Verifies the user's registration token
pub(super) async fn verify_registration_token<T: ServiceContract>(
    data: web::Query<EmailToken>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;
    info!("Verify registration token: {:?}", data);
    service.verify_registration_token(data.0).await
}

/// Resend the user's registration token in case it expired
pub(super) async fn resend_registration_token<T: ServiceContract>(
    data: web::Json<ResendRegToken>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;
    info!("Resend registration token: {:?}", data.0.email);
    service.resend_registration_token(data.0).await
}

/// Sets the user's OTP secret. Requires a valid session to be established beforehand
pub(super) async fn set_otp_secret<T: ServiceContract>(
    req: HttpRequest,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    let session = extract_session(req)?;
    info!("Registering OTP secret for: {}", session.user_id);
    service.set_otp_secret(&session.user_id).await
}

/// Verifies the user's OTP if they have 2FA enabled
pub(super) async fn verify_otp<T: ServiceContract>(
    data: web::Json<Otp>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;
    info!("OTP login : {:?}", data.0);
    service.verify_otp(data.0).await
}

/// Changes the user's password and purges all their sessions
pub(super) async fn change_password<T: ServiceContract>(
    data: web::Json<ChangePassword>,
    req: HttpRequest,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;
    let session = extract_session(req)?;
    info!("Updating password for {}", session.user_id);
    service.change_password(session, data.0).await
}

/// Sends a forgot password token via email
pub(super) async fn forgot_password<T: ServiceContract>(
    data: web::Json<ForgotPassword>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;
    info!("Forgot password, sending token to {}", data.0.email);
    service.forgot_password(data.0).await
}

/// Changes the user's password and purges all their sessions
pub(super) async fn verify_forgot_password<T: ServiceContract>(
    data: web::Json<ForgotPasswordVerify>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;
    info!("Forgot password, setting new");
    service.verify_forgot_password(data.0).await
}

/// Changes the user's password and purges all their sessions
pub(super) async fn reset_password<T: ServiceContract>(
    data: web::Query<ResetPassword>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;
    info!("Resetting password token: {:?}", data.0);
    service.reset_password(data.0).await
}

/// Logs the user out. Optionally purges their sessions, Requires a valid session to be established beforehand
pub(super) async fn logout<T: ServiceContract>(
    data: web::Json<Logout>,
    req: HttpRequest,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    let session = extract_session(req)?;
    info!("Logging out {}", session.user_id);
    service.logout(session, data.0).await
}
