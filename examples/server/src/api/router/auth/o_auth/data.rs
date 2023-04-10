use serde::{Deserialize, Serialize};
use validify::Validify;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) enum OAuthProvider {
    Google,
    Github,
}

#[derive(Debug, Clone, Deserialize, Validify)]
pub(super) struct OAuthCodeExchange {
    #[modify(trim)]
    #[validate(length(min = 1))]
    pub code: String,
}
