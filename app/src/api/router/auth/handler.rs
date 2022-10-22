use super::{
    contract::ServiceContract,
    data::{ChangePassword, Credentials, EmailToken, Logout, Otp, RegistrationData},
};
use crate::{error::Error, utility::request::extract_session};
use actix_web::{web, HttpRequest, Responder};
use tracing::info;
use validator::Validate;

/// Verifies the user's login credentials and either establishes a session if the user
/// doesn't have 2FA or prompts the user for their 2FA pass if they have it set up
pub(super) async fn verify_credentials<T: ServiceContract>(
    auth_form: web::Json<Credentials>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    auth_form.0.validate().map_err(Error::new)?;

    info!("Credentials login : {:?}", auth_form.0);

    service.verify_credentials(auth_form.0).await
}

/// Verifies the user's OTP if they have 2FA enabled
pub(super) async fn verify_otp<T: ServiceContract>(
    otp: web::Json<Otp>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    otp.0.validate().map_err(Error::new)?;

    info!("OTP login : {:?}", otp.0);

    service.verify_otp(otp.0).await
}

/// Starts the registration process for the user and sends an email containing a temporary
/// token used to complete the registration
pub(super) async fn start_registration<T: ServiceContract>(
    data: web::Form<RegistrationData>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;

    info!("Start Registration: {:?}", data.0);

    service.start_registration(data.0).await
}

/// Verifies the user's registration token
pub(super) async fn verify_registration_token<T: ServiceContract>(
    token: web::Query<EmailToken>,
    service: web::Data<T>,
) -> Result<impl Responder, Error> {
    token.0.validate().map_err(Error::new)?;

    info!("Verify registration token: {:?}", token);

    service.verify_registration_token(token.0).await
}

/// Sets the user's password after successful email token verification. Requires a token generated
/// after successful email verification
pub(super) async fn change_password<T: ServiceContract>(
    data: web::Form<ChangePassword>,
    service: web::Data<T>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    data.0.validate().map_err(Error::new)?;

    let session = extract_session(req)?;

    info!("Updating password for {}", session.user_id);
    service.change_password(&session.user_id, data.0).await
}

/// Sets the user's OTP secret. Requires a valid session to be established beforehand
pub(super) async fn set_otp_secret<T: ServiceContract>(
    service: web::Data<T>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    let session = extract_session(req)?;

    info!("Registering OTP secret for: {}", session.user_id);

    service.set_otp_secret(&session.user_id).await
}

/// Sets the user's OTP secret. Requires a valid session to be established beforehand
pub(super) async fn logout<T: ServiceContract>(
    service: web::Data<T>,
    expire: web::Json<Logout>,
    req: HttpRequest,
) -> Result<impl Responder, Error> {
    let session = extract_session(req)?;

    info!("Logging out {}", session.user_id);

    service
        .logout(&session.id, &session.user_id, expire.0)
        .await
}
