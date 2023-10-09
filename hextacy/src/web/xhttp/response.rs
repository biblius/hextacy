use cookie::Cookie;
use http::header;
use http::{
    header::{HeaderName, HeaderValue, InvalidHeaderValue},
    response::Builder,
    Response, StatusCode,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("Invalid header: {0}")]
    Header(#[from] InvalidHeaderValue),
    #[error("Http: {0}")]
    Http(#[from] http::Error),
    #[error("Serde: {0}")]
    Serde(#[from] serde_json::Error),
}

pub struct ResponseBuilder<T> {
    builder: Builder,
    body: T,
}

impl<T> ResponseBuilder<T> {
    pub fn with_cookies(
        mut self,
        cookies: &[Cookie<'_>],
    ) -> Result<ResponseBuilder<T>, ResponseError> {
        for cookie in cookies {
            self.builder = self.builder.header(
                header::SET_COOKIE,
                HeaderValue::try_from(cookie.to_string())?,
            );
        }

        Ok(self)
    }

    pub fn with_headers<K, V, const N: usize>(mut self, headers: [(K, V); N]) -> ResponseBuilder<T>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        for (key, value) in headers {
            self.builder = self.builder.header(key, value);
        }

        self
    }

    pub fn finish(self) -> Result<Response<T>, ResponseError> {
        Ok(self.builder.body(self.body)?)
    }
}

impl<T> ResponseBuilder<T>
where
    T: Serialize,
{
    /// Finish the response with a JSON body and set the content type to app/json if not already present.
    pub fn json(mut self) -> Result<Response<String>, ResponseError> {
        if let Some(headers) = self.builder.headers_ref() {
            if !headers.contains_key(header::CONTENT_TYPE) {
                self.builder = self
                    .builder
                    .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.essence_str())
            }
        }

        let json = serde_json::to_string(&self.body)?;

        self.builder.body(json).map_err(ResponseError::Http)
    }
}

/// Utility containing default methods for quickly converting a struct to an HTTP response.
pub trait RestResponse<'a>
where
    Self: Sized + Serialize,
{
    /// Enables quickly converting a struct to an http response with a JSON body and the provided cookies and headers.
    fn into_response(self, code: StatusCode) -> ResponseBuilder<Self> {
        ResponseBuilder {
            builder: Builder::new().status(code),
            body: self,
        }
    }
}
