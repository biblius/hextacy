use actix_web::{
    http::{
        header::{HeaderName, HeaderValue},
        StatusCode,
    },
    HttpResponse, HttpResponseBuilder,
};
use cookie::Cookie;
use serde::Serialize;

pub struct ResponseBuilder<'a, T: Response<'a>> {
    code: StatusCode,
    body: T,
    cookies: Vec<Cookie<'a>>,
    headers: Vec<(HeaderName, HeaderValue)>,
}

impl<'a, T> ResponseBuilder<'a, T>
where
    T: Response<'a>,
{
    pub fn with_cookies(mut self, cookies: Vec<Cookie<'a>>) -> ResponseBuilder<T> {
        for c in cookies {
            self.cookies.push(c);
        }
        self
    }

    pub fn with_headers(
        mut self,
        headers: Vec<(HeaderName, HeaderValue)>,
    ) -> ResponseBuilder<'a, T> {
        for h in headers {
            self.headers.push(h);
        }
        self
    }

    pub fn finish(self) -> HttpResponse {
        let mut response = HttpResponseBuilder::new(self.code);

        for c in self.cookies {
            response.cookie(c);
        }

        for (key, value) in self.headers {
            response.append_header((key, value));
        }

        response.json(self.body)
    }
}

/// Utility containing default methods for quickly converting a struct to an HTTP response
pub trait Response<'a>
where
    Self: Sized + Serialize,
{
    /// Enables quickly converting a struct to an http response with a JSON body and the provided cookies and headers.
    fn to_response(self, code: StatusCode) -> ResponseBuilder<'a, Self> {
        ResponseBuilder {
            code,
            body: self,
            cookies: vec![],
            headers: vec![],
        }
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

impl<'a> Response<'a> for MessageResponse<'a> {}
