use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EmailToken<'a>(&'a str);

impl<'a> EmailToken<'a> {
    pub fn token(&self) -> &'a str {
        self.0
    }
}
