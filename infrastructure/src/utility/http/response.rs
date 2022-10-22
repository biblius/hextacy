use actix_web::{
    http::{
        header::{HeaderName, HeaderValue},
        StatusCode,
    },
    HttpResponse, HttpResponseBuilder,
};
use cookie::Cookie;
use serde::Serialize;

pub trait Response
where
    Self: Sized + Serialize,
{
    /// Enables quickly converting a struct to an http response with a JSON body and the provided cookies.
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
