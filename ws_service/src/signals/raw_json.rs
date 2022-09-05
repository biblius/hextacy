use actix::Message;

#[derive(Message)]
#[rtype(result = "()")]
pub struct RawJson(pub String);

impl RawJson {
    pub fn to_inner(&self) -> String {
        self.0.clone()
    }

    pub fn get_inner(&self) -> &str {
        &self.0
    }
}
