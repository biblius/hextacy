use serde::Deserialize;
use validify::Validify;

#[derive(Debug, Clone, Deserialize, Validify)]
pub struct Register {
    #[modify(trim)]
    #[validate(length(min = 2))]
    pub username: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Clone, Deserialize, Validify)]
pub struct Login {
    #[validate(length(min = 1))]
    pub username: String,
    #[validate(length(min = 1))]
    pub password: String,
    pub remember: bool,
}

#[derive(Debug, Deserialize, Validify)]
pub struct Logout {
    pub purge: bool,
}
