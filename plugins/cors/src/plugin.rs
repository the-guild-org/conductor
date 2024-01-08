use crate::config::CorsPluginConfig;
use conductor_common::http::header::{
  HeaderValue, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS,
  ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_EXPOSE_HEADERS,
  ACCESS_CONTROL_MAX_AGE, ACCESS_CONTROL_REQUEST_HEADERS, CONTENT_LENGTH, ORIGIN, VARY,
};
use conductor_common::http::{HttpHeadersMap, Method};

use conductor_common::execute::RequestExecutionContext;
use conductor_common::http::{ConductorHttpResponse, StatusCode};
use conductor_common::plugin::{CreatablePlugin, Plugin, PluginError};

#[derive(Debug)]
pub struct CorsPlugin(CorsPluginConfig);

static WILDCARD: &str = "*";
static ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK: &str = "Access-Control-Allow-Private-Network";

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for CorsPlugin {
  type Config = CorsPluginConfig;

  async fn create(config: Self::Config) -> Result<Box<Self>, PluginError> {
    Ok(Box::new(Self(config)))
  }
}

impl CorsPlugin {
  /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin#browser_compatibility
  pub fn configure_origin(
    &self,
    request_headers: &HttpHeadersMap,
    response_headers: &mut HttpHeadersMap,
  ) {
    if let Some(origin) = &self.0.allowed_origin {
      let value = match origin.as_str() {
        "*" => WILDCARD,
        "reflect" => request_headers
          .get(ORIGIN)
          .and_then(|v| v.to_str().ok())
          .unwrap_or(WILDCARD),
        _ => origin,
      };

      if let Ok(parsed_value) = value.parse() {
        response_headers.append(ACCESS_CONTROL_ALLOW_ORIGIN, parsed_value);
      }
      if let Ok(vary_value) = "Origin".parse() {
        response_headers.append(VARY, vary_value);
      }
    }
  }

  /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials
  pub fn configure_credentials(&self, response_headers: &mut HttpHeadersMap) {
    if let Some(credentials) = &self.0.allow_credentials {
      if *credentials {
        if let Ok(parsed_value) = "true".parse() {
          response_headers.append(ACCESS_CONTROL_ALLOW_CREDENTIALS, parsed_value);
        }
      }
    }
  }

  /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Methods
  pub fn configure_methods(&self, response_headers: &mut HttpHeadersMap) {
    let value = self.0.allowed_methods.as_deref().unwrap_or(WILDCARD);
    if let Ok(parsed_value) = value.parse() {
      response_headers.append(ACCESS_CONTROL_ALLOW_METHODS, parsed_value);
    }
  }

  /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers
  pub fn configure_allowed_headers(
    &self,
    request_headers: &HttpHeadersMap,
    response_headers: &mut HttpHeadersMap,
  ) {
    match self.0.allowed_headers.as_deref() {
      None | Some("*") => {
        if let Some(source_header) = request_headers.get(ACCESS_CONTROL_REQUEST_HEADERS) {
          response_headers.append(ACCESS_CONTROL_ALLOW_HEADERS, source_header.clone());
          if let Ok(vary_value) = ACCESS_CONTROL_REQUEST_HEADERS.to_string().parse() {
            response_headers.append(VARY, vary_value);
          }
        }
      }
      Some(list) => {
        if let Ok(parsed_value) = list.parse() {
          response_headers.append(ACCESS_CONTROL_ALLOW_HEADERS, parsed_value);
        }
      }
    }
  }

  /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers
  pub fn configure_exposed_headers(&self, response_headers: &mut HttpHeadersMap) {
    if let Some(exposed_headers) = &self.0.exposed_headers {
      if let Ok(header_value) = HeaderValue::from_str(exposed_headers) {
        response_headers.insert(ACCESS_CONTROL_EXPOSE_HEADERS, header_value);
      }
    }
  }

  /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Max-Age
  pub fn configure_max_age(&self, response_headers: &mut HttpHeadersMap) {
    if let Some(max_age) = &self.0.max_age {
      if let Ok(header_value) = max_age.to_string().parse() {
        response_headers.insert(ACCESS_CONTROL_MAX_AGE, header_value);
      }
    }
  }

  pub fn configred_allow_private_netowkr(&self, response_headers: &mut HttpHeadersMap) {
    if let Some(allow_private_network) = &self.0.allow_private_network {
      if *allow_private_network {
        if let Ok(header_value) = "true".parse() {
          response_headers.insert(ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK, header_value);
        }
      }
    }
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for CorsPlugin {
  async fn on_downstream_http_request(&self, ctx: &mut RequestExecutionContext) {
    if ctx.downstream_http_request.method == Method::OPTIONS {
      let request_headers = &ctx.downstream_http_request.headers;
      let mut response_headers = HttpHeadersMap::new();
      self.configure_origin(request_headers, &mut response_headers);
      self.configure_credentials(&mut response_headers);
      self.configure_methods(&mut response_headers);
      self.configure_exposed_headers(&mut response_headers);
      self.configure_max_age(&mut response_headers);
      self.configure_allowed_headers(request_headers, &mut response_headers);

      if let Ok(content_length_value) = "0".parse() {
        response_headers.insert(CONTENT_LENGTH, content_length_value);
      }

      ctx.short_circuit(ConductorHttpResponse {
        status: StatusCode::OK,
        headers: response_headers,
        body: Default::default(),
      })
    }
  }

  fn on_downstream_http_response(
    &self,
    ctx: &mut RequestExecutionContext,
    response: &mut ConductorHttpResponse,
  ) {
    let request_headers = &ctx.downstream_http_request.headers;
    self.configure_origin(request_headers, &mut response.headers);
    self.configure_credentials(&mut response.headers);
    self.configure_exposed_headers(&mut response.headers);
  }
}
