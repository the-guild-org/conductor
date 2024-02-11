use conductor_common::http::{header::*, StatusCode};
use conductor_tracing::{otel_attrs::*, trace_id::generate_trace_id};
use minitrace::{
  collector::{SpanContext, SpanId},
  Span,
};
use worker::*;

#[inline]
fn strip_http_flavor(value: &String) -> String {
  value.split('/').nth(1).unwrap_or(value).to_owned()
}

#[inline]
fn build_request_properties(req: &Request) -> Vec<(&'static str, String)> {
  let headers = req.headers();
  let user_agent = headers
    .get(USER_AGENT.as_str())
    // @expected: it panics only if the header name is not valid, and we know it is.
    .unwrap()
    .unwrap_or_default();
  // @expected: it panics only if the URL source is not valid, and it's already validated before.
  let url = req.url().unwrap();
  let http_route = url.path();
  let http_method = req.method().to_string();
  // @expected: it only panics if we are not running in a CF context, should be safe.
  let cf_info = req.cf().unwrap();
  let request_id = headers
    .get("x-request-id")
    .unwrap()
    .or_else(|| headers.get("cf-ray").unwrap())
    .unwrap_or_default();
  let http_flavor = strip_http_flavor(&cf_info.http_protocol());
  let http_scheme = url.scheme().to_owned();
  // @expected: unwraps only in special cases where "data:text" is used.
  let http_host = url.host().unwrap().to_string();
  // See https://developers.cloudflare.com/fundamentals/reference/http-request-headers/#true-client-ip-enterprise-plan-only
  let client_id = headers
    .get("true-client-ip")
    .unwrap()
    .or_else(|| headers.get("cf-connecting-ip").unwrap())
    .unwrap_or_default();

  let http_target = format!(
    "{}{}",
    http_route,
    url.query().map(|v| format!("?{}", v)).unwrap_or_default()
  );

  vec![
    (HTTP_METHOD, http_method),
    (HTTP_ROUTE, http_route.to_string()),
    (HTTP_FLAVOR, http_flavor),
    (HTTP_SCHEME, http_scheme),
    (HTTP_HOST, http_host),
    (HTTP_CLIENT_IP, client_id),
    (HTTP_USER_AGENT, user_agent),
    (HTTP_TARGET, http_target),
    (OTEL_KIND, "server".to_string()),
    (REQUEST_ID, request_id),
    // Specific to DataDog
    (SPAN_TYPE, "web".to_string()),
  ]
}

#[inline]
pub fn build_request_root_span(tenant_id: u32, endpoint_identifier: &str, req: &Request) -> Span {
  let method = req.method().to_string();
  let span_name = format!("HTTP {} {}", method, req.path());
  let mut properties: Vec<(&str, String)> = build_request_properties(req);
  properties.push((CONDUCTOR_ENDPOINT, endpoint_identifier.to_owned()));

  let span_context = SpanContext::new(generate_trace_id(tenant_id), SpanId::default());

  Span::root(span_name, span_context).with_properties(|| properties)
}

#[inline]
fn handle_error(response_error: &worker::Error) -> Vec<(&'static str, String)> {
  let mut properties: Vec<(&'static str, String)> = vec![];

  let display = format!("{response_error}");
  let debug = format!("{response_error:?}");
  properties.push((EXCEPTION_MESSAGE, display));
  properties.push((EXCEPTION_DETAILS, debug));
  properties.push((ERROR_INDICATOR, "true".to_string()));

  properties
}

#[inline]
pub fn build_response_properties(res: &Result<Response>) -> Vec<(&'static str, String)> {
  let mut properties: Vec<(&'static str, String)> = vec![];

  match res {
    Ok(res) => {
      let status_code = StatusCode::from_u16(res.status_code()).unwrap();

      if status_code == StatusCode::OK || status_code.is_client_error() {
        properties.push((OTEL_STATUS_CODE, "OK".to_string()));
      } else {
        properties.push((OTEL_STATUS_CODE, "ERROR".to_string()));
      }

      properties.push((HTTP_STATUS_CODE, status_code.as_u16().to_string()));
    }
    Err(e) => {
      properties.append(&mut handle_error(e));
      properties.push((
        HTTP_STATUS_CODE,
        StatusCode::INTERNAL_SERVER_ERROR.as_u16().to_string(),
      ));
    }
  }

  properties
}
