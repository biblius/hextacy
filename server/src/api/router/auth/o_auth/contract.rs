use super::data::OAuthCodeExchange;
use crate::error::Error;
use actix_web::HttpResponse;
use alx_core::clients::oauth::{OAuthAccount, TokenResponse};
use async_trait::async_trait;
use storage::models::{session::Session, user::User};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub(super) trait ServiceContract {
    /// Process the code received in the authorization step and log the user in or auto
    /// register them, based on whether they already exist. Establishes a session.
    ///
    /// We support incremental authorization, therefore we need to check
    /// existing oauth entries since there's a chance users already granted
    /// more scopes in their previous sessions. If that's the case, we refresh
    /// the existing ones using the refresh token and establish a session based
    /// on that, as we always want to keep only a single entry per user and provider
    /// in the `oauth` table. Multiple sessions with the same access token are allowed.
    async fn login(&self, code: OAuthCodeExchange) -> Result<HttpResponse, Error>;

    /// Auto register the user based on the obtained OAuth account from the login step.
    fn register<T, A>(&self, tokens: T, account: A) -> Result<HttpResponse, Error>
    where
        T: TokenResponse + Send + Sync + 'static,
        A: OAuthAccount + Send + Sync + 'static;

    /// Mainly used for incremental authorization. When the user wants to perform an action
    /// not permitted by their current scopes, the frontend should perform another authorization request
    /// with additional scopes and send the code here to exchange it for a token. The newly obtained token
    /// should replace the old one, as it will contain all the previously granted scopes and the session
    /// (and cookies) should be updated to reflect the change.
    async fn request_additional_scopes(
        &self,
        mut session: Session,
        code: OAuthCodeExchange,
    ) -> Result<HttpResponse, Error>;

    fn establish_session<T>(&self, tokens: T, user: User) -> Result<HttpResponse, Error>
    where
        T: TokenResponse + Send + Sync + 'static;
}
