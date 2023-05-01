use serde::Deserialize;
use validify::Validify;

#[derive(Debug, Clone, Deserialize, Validify)]
pub(super) struct OAuthCodeExchange {
    #[modify(trim)]
    #[validate(length(min = 1))]
    pub code: String,
}
