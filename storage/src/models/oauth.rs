use chrono::NaiveDateTime;
use diesel::Queryable;

/// Entry in the `oauth` table.
///
/// #### A note on revoking:
///
/// Entries revoked on the backend only influence the app's ability
/// to find them in the DB, the tokens may not be revoked on the provider. Always make
/// sure when revoking to actually send a revokation request to the provider.
#[derive(Debug, Clone, Queryable)]
pub struct OAuthMeta {
    pub id: String,
    pub user_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub provider: String,
    pub account_id: String,
    pub scope: String,
    pub revoked: bool,
    pub expires_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl OAuthMeta {
    pub fn expired(&self) -> bool {
        if let Some(expiration) = self.expires_at {
            if utils::time::datetime_now() >= expiration {
                return true;
            }
        }
        false
    }
}
