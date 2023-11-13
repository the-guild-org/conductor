use std::collections::HashMap;

pub use bytes::Bytes;
pub use http::Uri;
use http::{HeaderMap, StatusCode as RawStatusCode};
pub use url::Url;

pub use http::header::{HeaderValue, ACCEPT, CONTENT_TYPE};
pub use http::Method;
pub use mime::{Mime, APPLICATION_JSON, APPLICATION_WWW_FORM_URLENCODED};
use serde::de::DeserializeOwned;
use serde_json::from_slice;
pub type StatusCode = RawStatusCode;
pub type HttpHeadersMap = HeaderMap<HeaderValue>;

#[derive(Debug)]
pub struct ConductorHttpRequest {
    pub headers: HeaderMap<HeaderValue>,
    pub method: Method,
    pub uri: String,
    pub query_string: String,
    pub body: Bytes,
}

impl ConductorHttpRequest {
    pub fn json_body<T>(&self) -> Result<T, serde_json::Error>
    where
        T: DeserializeOwned,
    {
        from_slice::<T>(&self.body)
    }
}

#[derive(Debug)]
pub struct ConductorHttpResponse {
    pub body: Bytes,
    pub status: StatusCode,
    pub headers: HttpHeadersMap,
}

pub fn extract_content_type(headers_map: &HttpHeadersMap) -> Option<Mime> {
    let content_type = headers_map
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    content_type.and_then(|content_type| content_type.parse().ok())
}

pub fn extract_accept(headers_map: &HeaderMap) -> Option<Mime> {
    let content_type = headers_map
        .get(ACCEPT)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    content_type.and_then(|content_type| content_type.parse().ok())
}

pub fn parse_query_string(input: &str) -> HashMap<String, String> {
    querystring::querify(input)
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}
