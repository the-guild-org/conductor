use std::str::FromStr;

use conductor_common::{
  graphql::GraphQLResponse,
  http::{ConductorHttpRequest, HeaderName, HeaderValue, Method, StatusCode},
  logging_locks::RwLockWriteGuard,
  vrl_functions::ShortCircuitFn,
  vrl_utils::conductor_request_to_value,
};
use tracing::error;
use vrl::{
  compiler::{Context, Program, TargetValue, TimeZone},
  value,
  value::{Secrets, Value},
};

use conductor_common::execute::RequestExecutionContext;

static METADATA_UPSTREAM_HTTP_REQ: &str = "upstream_http_req";
static TARGET_UPSTREAM_HTTP_REQ_VALUE_HEADERS: &str = "upstream_http_req.headers";
static TARGET_UPSTREAM_HTTP_REQ_VALUE_METHOD: &str = "upstream_http_req.method";
static TARGET_UPSTREAM_HTTP_REQ_VALUE_URI: &str = "upstream_http_req.uri";
static TARGET_UPSTREAM_HTTP_REQ_VALUE_QUERY_STRING: &str = "upstream_http_req.query_string";
static TARGET_UPSTREAM_HTTP_REQ_VALUE_BODY: &str = "upstream_http_req.body";

pub fn vrl_upstream_http_request(
  program: &Program,
  ctx: &mut RwLockWriteGuard<'_, RequestExecutionContext>,
  req: &mut ConductorHttpRequest,
) {
  let upstream_req_value = conductor_request_to_value(req);
  let mut target = TargetValue {
    value: value!({}),
    metadata: value!({}),
    secrets: Secrets::default(),
  };

  target.value.insert(
    TARGET_UPSTREAM_HTTP_REQ_VALUE_HEADERS,
    Value::Object(Default::default()),
  );
  target
    .value
    .insert(TARGET_UPSTREAM_HTTP_REQ_VALUE_METHOD, Value::Null);
  target
    .value
    .insert(TARGET_UPSTREAM_HTTP_REQ_VALUE_URI, Value::Null);
  target
    .value
    .insert(TARGET_UPSTREAM_HTTP_REQ_VALUE_QUERY_STRING, Value::Null);
  target
    .metadata
    .insert(METADATA_UPSTREAM_HTTP_REQ, upstream_req_value);

  match program.resolve(&mut Context::new(
    &mut target,
    ctx.vrl_shared_state(),
    &TimeZone::default(),
  )) {
    Ok(ret) => {
      if let Some((error_code, message)) = ShortCircuitFn::check_short_circuit(&ret) {
        ctx.short_circuit(
          GraphQLResponse::new_error(&String::from_utf8_lossy(&message)).into_with_status_code(
            StatusCode::from_u16(error_code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
          ),
        );

        return;
      }

      if let Some(Value::Object(headers)) = target
        .value
        .remove(TARGET_UPSTREAM_HTTP_REQ_VALUE_HEADERS, false)
      {
        for (k, v) in headers {
          match v {
            Value::Bytes(b) => {
              if let Ok(name) = HeaderName::from_str(&k) {
                if let Ok(value) = HeaderValue::from_bytes(&b) {
                  req.headers.insert(name, value);
                } else {
                  error!("couldn't create header value from the provided string!")
                }
              } else {
                error!("couldn't create header key from the provided string!")
              }
            }
            Value::Null => {
              if let Ok(header_key) = HeaderName::from_str(&k) {
                req.headers.remove(header_key);
              } else {
                error!("couldn't remove header with the provided key: {:?}", k)
              }
            }
            _ => {}
          }
        }
      }

      if let Some(Value::Bytes(method)) = target
        .value
        .remove(TARGET_UPSTREAM_HTTP_REQ_VALUE_METHOD, false)
      {
        if let Ok(method) = Method::from_bytes(&method) {
          req.method = method;
        } else {
          error!("couldn't retrieve the method of the http request!")
        }
      }

      if let Some(Value::Bytes(uri)) = target
        .value
        .remove(TARGET_UPSTREAM_HTTP_REQ_VALUE_URI, false)
      {
        if let Ok(uri) = String::from_utf8(uri.into()) {
          req.uri = uri;
        } else {
          error!("couldn't retrieve the uri of the http request!")
        }
      }

      if let Some(Value::Bytes(query_string)) = target
        .value
        .remove(TARGET_UPSTREAM_HTTP_REQ_VALUE_QUERY_STRING, false)
      {
        if let Ok(query_string) = String::from_utf8(query_string.into()) {
          req.query_string = query_string;
        } else {
          error!("couldn't retrieve the query_string of the http request!")
        }
      }

      if let Some(Value::Bytes(body)) = target
        .value
        .remove(TARGET_UPSTREAM_HTTP_REQ_VALUE_BODY, false)
      {
        req.body = body;
      }
    }
    Err(err) => {
      error!("vrl::upstream_http_request resolve error: {:?}", err);

      ctx.short_circuit(
        GraphQLResponse::new_error("vrl runtime error")
          .into_with_status_code(StatusCode::BAD_GATEWAY),
      );
    }
  }
}
