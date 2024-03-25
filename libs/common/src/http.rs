use std::collections::HashMap;
use std::str::FromStr;

use anyhow::{anyhow, Result as AnyhowResult};
pub use bytes::Bytes;
pub use http::Uri;
use http::{HeaderMap, StatusCode as RawStatusCode};
use serde::{Deserialize, Serialize};
pub use url::Url;

pub use http::header;
pub use http::header::{HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE};
pub use http::Method;
pub use mime::{Mime, APPLICATION_JSON, APPLICATION_WWW_FORM_URLENCODED};
use serde::de::DeserializeOwned;
use serde_json::from_slice;
pub type StatusCode = RawStatusCode;
pub type HttpHeadersMap = HeaderMap<HeaderValue>;

pub trait ToHeadersMap {
  fn to_headers_map(&self) -> AnyhowResult<HttpHeadersMap>;
}

impl ToHeadersMap for HashMap<String, String> {
  fn to_headers_map(&self) -> Result<HttpHeadersMap, anyhow::Error> {
    let mut headers_map = HeaderMap::new();

    for (key, value) in self {
      let header_name = HeaderName::from_str(key)
        .map_err(|e| anyhow!("Couldn't parse key into a header name: {}", e))?;
      let header_value = HeaderValue::from_str(value)
        .map_err(|e| anyhow!("Couldn't parse value into a header value: {}", e))?;

      headers_map.insert(header_name, header_value);
    }

    Ok(headers_map)
  }
}

impl ToHeadersMap for Vec<(&str, &str)> {
  fn to_headers_map(&self) -> Result<HttpHeadersMap, anyhow::Error> {
    let mut headers_map = HeaderMap::new();

    for (key, value) in self {
      let header_name = HeaderName::from_str(key)
        .map_err(|e| anyhow!("Couldn't parse key into a header name: {}", e))?;
      let header_value = HeaderValue::from_str(value)
        .map_err(|e| anyhow!("Couldn't parse value into a header value: {}", e))?;

      headers_map.insert(header_name, header_value);
    }

    Ok(headers_map)
  }
}

#[derive(Debug, Clone)]
pub struct ConductorHttpRequest {
  pub headers: HeaderMap<HeaderValue>,
  pub method: Method,
  pub uri: String,
  pub query_string: String,
  pub body: Bytes,
}

#[cfg(feature = "test_utils")]
impl Default for ConductorHttpRequest {
  fn default() -> Self {
    Self {
      headers: HeaderMap::new(),
      method: Method::GET,
      uri: "/".to_string(),
      query_string: "".to_string(),
      body: serde_json::json!({
          "query": "query { __typename }",
      })
      .to_string()
      .into(),
    }
  }
}

impl ConductorHttpRequest {
  pub fn json_body<T>(&self) -> Result<T, serde_json::Error>
  where
    T: DeserializeOwned,
  {
    from_slice::<T>(&self.body)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConductorHttpResponse {
  pub body: Bytes,
  #[serde(with = "http_serde::status_code")]
  pub status: StatusCode,
  #[serde(with = "http_serde::header_map")]
  pub headers: HeaderMap,
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
