use actix_web::{
    http::{
        header::{HeaderName, HeaderValue},
        StatusCode,
    },
    HttpResponse, HttpResponseBuilder,
};
use cookie::Cookie;
use serde::Serialize;

/// Utility containing default methods for quickly converting a struct to an HTTP response
pub trait Response
where
    Self: Sized + Serialize,
{
    /// Enables quickly converting a struct to an http response with a JSON body and the provided cookies and headers.
    fn to_response(
        self,
        code: StatusCode,
        cookies: Option<Vec<Cookie<'_>>>,
        headers: Option<Vec<(HeaderName, HeaderValue)>>,
    ) -> HttpResponse {
        let mut response = HttpResponseBuilder::new(code);
        if let Some(cookies) = cookies {
            for c in cookies {
                response.cookie(c);
            }
        }
        if let Some(headers) = headers {
            for (key, value) in headers {
                response.append_header((key, value));
            }
        }
        response.json(self)
    }
}

/// Holds a single message. Implements the Response trait.
#[derive(Debug, Serialize)]
pub struct MessageResponse<'a> {
    message: &'a str,
}

impl<'a> MessageResponse<'a> {
    pub fn new(message: &'a str) -> Self {
        Self { message }
    }
}

impl<'a> Response for MessageResponse<'a> {}
