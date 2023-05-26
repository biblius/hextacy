use super::adapters::{
    cache::AuthenticationCacheContract, repository::AuthenticationRepositoryContract,
};
use super::data::OAuthCodeExchange;
use crate::{
    app::core::auth::data::AuthenticationSuccessResponse,
    config::constants::COOKIE_S_ID,
    error::{AuthenticationError, Error},
    services::oauth::{OAuthAccount, OAuthProvider, TokenResponse},
};
use crate::{
    db::models::{session::Session, user::User},
    services::oauth::OAuth,
};
use actix_web::HttpResponse;
use hextacy::{
    contract,
    crypto::uuid,
    web::http::response::{MessageResponse, Response},
};
use reqwest::{
    header::{HeaderName, HeaderValue},
    StatusCode,
};
use tracing::info;

pub struct OAuthService<R, C>
where
    R: AuthenticationRepositoryContract,
    C: AuthenticationCacheContract,
{
    pub repository: R,
    pub cache: C,
}

#[contract]
impl<R, C> OAuthService<R, C>
where
    R: AuthenticationRepositoryContract + Send + Sync,
    C: AuthenticationCacheContract + Send + Sync,
{
    /// Process the code received in the authorization step and log the user in or auto
    /// register them, based on whether they already exist. Establishes a session.
    ///
    /// We support incremental authorization, therefore we need to check
    /// existing oauth entries since there's a chance users already granted
    /// more scopes in their previous sessions. If that's the case, we refresh
    /// the existing ones using the refresh token and establish a session based
    /// on that, as we always want to keep only a single entry per user and provider
    /// in the `oauth` table. Multiple sessions with the same access token are allowed.
    async fn login<T: OAuth + Send + Sync + 'static>(
        &self,
        provider: T,
        code: OAuthCodeExchange,
    ) -> Result<HttpResponse, Error> {
        let OAuthCodeExchange { ref code } = code;

        // Get the tokens and obtain the account
        let mut tokens = provider.exchange_code(code).await?;
        let account = provider.get_account(&tokens).await?;
        let provider_id = provider.provider_id();
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
                provider_id,
            )
            .await?;

        if oauth.expired() {
            if let Some(ref refresh_token) = oauth.refresh_token {
                info!("OAuth access token expired, refreshing");
                tokens = provider.refresh_access_token(refresh_token).await?;
                self.repository
                    .refresh_oauth_and_session(&user.id, &tokens, provider_id)
                    .await?;
            } else {
                self.repository
                    .update_oauth(&user.id, &tokens, provider_id)
                    .await?;
            }
        }

        self.establish_session(provider_id, tokens, user).await
    }

    /// Mainly used for incremental authorization. When the user wants to perform an action
    /// not permitted by their current scopes, the frontend should perform another authorization request
    /// with additional scopes and send the code here to exchange it for a token. The newly obtained token
    /// should replace the old one, as it will contain all the previously granted scopes and the session
    /// (and cookies) should be updated to reflect the change.
    async fn request_additional_scopes<T: OAuth + Send + Sync + 'static>(
        &self,
        provider: T,
        mut session: Session,
        code: OAuthCodeExchange,
    ) -> Result<HttpResponse, Error> {
        let _ = session
            .oauth_token
            .ok_or(AuthenticationError::InvalidToken("OAuth"))?;

        // Obtain the new tokens with more scopes
        let OAuthCodeExchange { ref code } = code;
        let tokens = provider.exchange_code(code).await?;

        let user_id = &session.user_id;
        let provider_id = provider.provider_id();
        let access_token = tokens.access_token();

        // Update existing sessions tied to the user and their auth provider
        // as well as the related oauth metadata
        self.repository
            .update_session_access_tokens(access_token, user_id, provider_id)
            .await?;
        self.repository
            .update_oauth(user_id, &tokens, provider_id)
            .await?;

        // Update the existing session, sessions updated in the previous step will not update
        // cached sessions so we have to cache the current one to reflect the change
        session.oauth_token = Some(access_token.to_string());

        self.cache.set_session(&session.id, &session).await?;

        Ok(MessageResponse::new("lol")
            .to_response(StatusCode::OK)
            .finish())
    }

    async fn establish_session<TR: TokenResponse + 'static>(
        &self,
        provider_id: OAuthProvider,
        tokens: TR,
        user: User,
    ) -> Result<HttpResponse, Error> {
        let csrf_token = uuid().to_string();

        let expiration = tokens.expires_in();
        let access_token = tokens.access_token();

        let session = self
            .repository
            .create_session(
                &user,
                &csrf_token,
                expiration,
                Some(access_token),
                Some(provider_id),
            )
            .await?;

        let session_cookie = crate::helpers::cookie::create(COOKIE_S_ID, &session.id, false)?;

        // Cache the session
        self.cache.set_session(&session.id, &session).await?;

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
/*
#[async_trait]
impl<R, C> ServiceApi for OAuthService<R, C>
where
    R: AuthenticationRepositoryContract + Send + Sync,
    C: AuthenticationCacheContract + Send + Sync,
{
    async fn login<T: OAuth + Send + Sync>(
        &self,
        provider: T,
        code: OAuthCodeExchange,
    ) -> Result<HttpResponse, Error> {
        let OAuthCodeExchange { ref code } = code;

        // Get the tokens and obtain the account
        let mut tokens = provider.exchange_code(code).await?;
        let account = provider.get_account(&tokens).await?;
        let provider_id = provider.provider_id();
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
                provider_id,
            )
            .await?;

        if oauth.expired() {
            if let Some(ref refresh_token) = oauth.refresh_token {
                info!("OAuth access token expired, refreshing");
                tokens = provider.refresh_access_token(refresh_token).await?;
                self.repository
                    .refresh_oauth_and_session(&user.id, &tokens, provider_id)
                    .await?;
            } else {
                self.repository
                    .update_oauth(&user.id, &tokens, provider_id)
                    .await?;
            }
        }

        self.establish_session(provider_id, tokens, user).await
    }

    async fn request_additional_scopes<T: OAuth + Send + Sync>(
        &self,
        provider: T,
        mut session: Session,
        code: OAuthCodeExchange,
    ) -> Result<HttpResponse, Error> {
        let _ = session
            .oauth_token
            .ok_or(AuthenticationError::InvalidToken("OAuth"))?;

        // Obtain the new tokens with more scopes
        let OAuthCodeExchange { ref code } = code;
        let tokens = provider.exchange_code(code).await?;

        let user_id = &session.user_id;
        let provider_id = provider.provider_id();
        let access_token = tokens.access_token();

        // Update existing sessions tied to the user and their auth provider
        // as well as the related oauth metadata
        self.repository
            .update_session_access_tokens(access_token, user_id, provider_id)
            .await?;
        self.repository
            .update_oauth(user_id, &tokens, provider_id)
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
        provider_id: OAuthProvider,
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
                Some(provider_id),
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
 */
