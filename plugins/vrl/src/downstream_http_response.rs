use std::str::FromStr;

use conductor_common::{
  graphql::GraphQLResponse,
  http::{ConductorHttpResponse, HeaderName, HeaderValue, StatusCode},
  vrl_functions::ShortCircuitFn,
  vrl_utils::conductor_response_to_value,
};
use tracing::error;
use vrl::{
  compiler::{Context, Program, TargetValue, TimeZone},
  value,
  value::{Secrets, Value},
};

use conductor_common::execute::RequestExecutionContext;

static METADATA_DOWNSTREAM_HTTP_RES: &str = "downstream_http_res";
static TARGET_DOWNSTREAM_HTTP_RES_VALUE_HEADERS: &str = "downstream_http_res.headers";
static TARGET_DOWNSTREAM_HTTP_RES_VALUE_STATUS: &str = "downstream_http_res.status";
static TARGET_DOWNSTREAM_HTTP_RES_VALUE_BODY: &str = "downstream_http_res.body";

pub fn vrl_downstream_http_response(
  program: &Program,
  ctx: &mut RequestExecutionContext,
  response: &mut ConductorHttpResponse,
) {
  let downstream_res_value = conductor_response_to_value(response);

  let mut target = TargetValue {
    value: value!({}),
    metadata: value!({}),
    secrets: Secrets::default(),
  };
  target.value.insert(
    TARGET_DOWNSTREAM_HTTP_RES_VALUE_HEADERS,
    Value::Object(Default::default()),
  );
  target
    .value
    .insert(TARGET_DOWNSTREAM_HTTP_RES_VALUE_STATUS, Value::Null);
  target
    .value
    .insert(TARGET_DOWNSTREAM_HTTP_RES_VALUE_BODY, Value::Null);
  target
    .metadata
    .insert(METADATA_DOWNSTREAM_HTTP_RES, downstream_res_value);

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
        .remove(TARGET_DOWNSTREAM_HTTP_RES_VALUE_HEADERS, false)
      {
        for (k, v) in headers {
          match v {
            Value::Bytes(b) => {
              if let Ok(name) = HeaderName::from_str(&k) {
                if let Ok(value) = HeaderValue::from_bytes(&b) {
                  response.headers.insert(name, value);
                } else {
                  error!("couldn't create header value from the provided string!")
                }
              } else {
                error!("couldn't create header key from the provided string!")
              }
            }
            Value::Null => {
              if let Ok(header_key) = HeaderName::from_str(&k) {
                response.headers.remove(header_key);
              } else {
                error!("couldn't remove header with the provided key: {:?}", k)
              }
            }
            _ => {}
          }
        }
      }

      if let Some(Value::Integer(status)) = target
        .value
        .remove(TARGET_DOWNSTREAM_HTTP_RES_VALUE_STATUS, false)
      {
        response.status =
          StatusCode::from_u16(status as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
      }

      if let Some(Value::Bytes(body)) = target
        .value
        .remove(TARGET_DOWNSTREAM_HTTP_RES_VALUE_BODY, false)
      {
        response.body = body;
      }
    }
    Err(err) => {
      error!("vrl::on_downstream_response resolve error: {:?}", err);

      ctx.short_circuit(
        GraphQLResponse::new_error("vrl runtime error")
          .into_with_status_code(StatusCode::BAD_GATEWAY),
      );
    }
  }
}
