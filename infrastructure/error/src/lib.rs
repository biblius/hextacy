use thiserror;

pub enum Error {}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}
impl Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "There was an error: {}", self)
    }
}

impl ResponseError for ErrorResponse {
    fn status_code(&self) -> reqwest::StatusCode {
        match self {
            Self::AuthenticationError(e) => e.status_code(),
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    fn error_response(&self) -> HttpResponse<BoxBody> {
        let status = self.status_code();
        let error_response = ErrorResponse {
            code: status.as_u16(),
            error: self.to_string(),
            message: self.message(),
        };
        Response::new(status).json(error_response)
    }
}
