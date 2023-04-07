use super::{api::ServiceApi, data::OAuthCodeExchange};
use crate::{
    api::router::auth::{
        api::{CacheApi, RepositoryApi},
        data::AuthenticationSuccessResponse,
    },
    config::constants::COOKIE_S_ID,
    error::{AuthenticationError, Error},
    services::oauth::{OAuthAccount, TokenResponse},
};
use crate::{
    db::models::{session::Session, user::User},
    services::oauth::OAuth,
};
use actix_web::HttpResponse;
use async_trait::async_trait;
use hextacy::{
    crypto::uuid,
    web::http::response::{MessageResponse, Response},
};
use reqwest::{
    header::{HeaderName, HeaderValue},
    StatusCode,
};
use tracing::info;

pub(super) struct OAuthService<P, R, C>
where
    P: OAuth,
    R: RepositoryApi,
    C: CacheApi,
{
    pub provider: P,
    pub repository: R,
    pub cache: C,
}

#[async_trait]
impl<P, R, C> ServiceApi for OAuthService<P, R, C>
where
    P: OAuth + Send + Sync,
    R: RepositoryApi + Send + Sync,
    C: CacheApi + Send + Sync,
{
    async fn login(&self, code: OAuthCodeExchange) -> Result<HttpResponse, Error> {
        let OAuthCodeExchange { ref code } = code;

        // Get the tokens and obtain the account
        let mut tokens = self.provider.exchange_code(code).await?;
        let account = self.provider.get_account(&tokens).await?;
        let provider = self.provider.provider_id();
        let email = match account.email() {
            Some(email) => email,
            None => return Err(AuthenticationError::EmailUnverified.into()),
        };

        let account_id = account.id();

        let (user, oauth) = self
            .repository
            .get_or_create_user_oauth(
                account_id.as_str(),
                email,
                account.username(),
                &tokens,
                provider,
            )
            .await?;

        if oauth.expired() {
            if let Some(ref refresh_token) = oauth.refresh_token {
                info!("OAuth access token expired, refreshing");
                tokens = self.provider.refresh_access_token(refresh_token).await?;
                self.repository
                    .refresh_oauth_and_session(&user.id, &tokens, provider)
                    .await?;
            } else {
                self.repository
                    .update_oauth(&user.id, &tokens, provider)
                    .await?;
            }
        }

        self.establish_session(tokens, user).await
    }

    async fn request_additional_scopes(
        &self,
        mut session: Session,
        code: OAuthCodeExchange,
    ) -> Result<HttpResponse, Error> {
        let _ = session
            .oauth_token
            .ok_or_else(|| AuthenticationError::InvalidToken("OAuth"))?;

        // Obtain the new tokens with more scopes
        let OAuthCodeExchange { ref code } = code;
        let tokens = self.provider.exchange_code(code).await?;

        let user_id = &session.user_id;
        let provider = self.provider.provider_id();
        let access_token = tokens.access_token();

        // Update existing sessions tied to the user and their auth provider
        // as well as the related oauth metadata
        self.repository
            .update_session_access_tokens(access_token, user_id, provider)
            .await?;
        self.repository
            .update_oauth(user_id, &tokens, provider)
            .await?;

        // Update the existing session, sessions updated in the previous step will not update
        // cached sessions so we have to cache the current one to reflect the change
        session.oauth_token = Some(access_token.to_string());

        self.cache.set_session(&session.id, &session)?;

        Ok(MessageResponse::new("lol")
            .to_response(StatusCode::OK)
            .finish())
    }

    async fn establish_session<TR: TokenResponse>(
        &self,
        tokens: TR,
        user: User,
    ) -> Result<HttpResponse, Error> {
        let csrf_token = uuid();

        let expiration = tokens.expires_in();
        let access_token = tokens.access_token();

        let session = self
            .repository
            .create_session(
                &user,
                &csrf_token,
                expiration,
                Some(access_token),
                Some(self.provider.provider_id()),
            )
            .await?;

        let session_cookie = crate::helpers::cookie::create(COOKIE_S_ID, &session.id, false)?;

        // Cache the session
        self.cache.set_session(&session.id, &session)?;

        info!("Successfully created session for {}", user.username);

        // Respond with the x-csrf header and the session ID
        Ok(AuthenticationSuccessResponse::new(user)
            .to_response(StatusCode::OK)
            .with_cookies(vec![session_cookie])
            .with_headers(vec![(
                HeaderName::from_static("x-csrf-token"),
                HeaderValue::from_str(&csrf_token)?,
            )])
            .finish())
    }
}
