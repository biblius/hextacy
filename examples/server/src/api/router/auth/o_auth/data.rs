use serde::{Deserialize, Serialize};
use validify::validify;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) enum OAuthProvider {
    Google,
    Github,
}

#[derive(Debug, Clone, Deserialize)]
#[validify]
pub(super) struct OAuthCodeExchange {
    #[modify(trim)]
    #[validate(length(min = 1))]
    pub code: String,
}
