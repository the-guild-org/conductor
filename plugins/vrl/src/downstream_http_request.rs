use conductor_common::{
  graphql::{GraphQLRequest, GraphQLResponse, ParsedGraphQLRequest},
  http::StatusCode,
  vrl_functions::ShortCircuitFn,
  vrl_utils::{conductor_request_to_value, vrl_value_to_serde_value},
};
use tracing::error;
use vrl::{
  compiler::{Context, Program, TargetValue, TimeZone},
  value,
  value::{Secrets, Value},
};

use conductor_common::execute::RequestExecutionContext;

static TARGET_GRAPHQL_OPERATION_KEY: &str = "graphql.operation";
static TARGET_GRAPHQL_OPERATION_NAME: &str = "graphql.operation_name";
static TARGET_GRAPHQL_OPERATION_VARIABLES: &str = "graphql.variables";
static TARGET_GRAPHQL_OPERATION_EXTENSIONS: &str = "graphql.extensions";
static METADATA_DOWNSTREAM_HTTP_REQUEST: &str = "downstream_http_req";

pub fn vrl_downstream_http_request(program: &Program, ctx: &mut RequestExecutionContext) {
  let downstream_req_value = conductor_request_to_value(&ctx.downstream_http_request);
  let mut target = TargetValue {
    value: value!({}),
    metadata: value!({}),
    secrets: Secrets::default(),
  };

  target
    .value
    .insert(TARGET_GRAPHQL_OPERATION_KEY, Value::Null);
  target
    .value
    .insert(TARGET_GRAPHQL_OPERATION_NAME, Value::Null);
  target.value.insert(
    TARGET_GRAPHQL_OPERATION_VARIABLES,
    Value::Object(Default::default()),
  );
  target.value.insert(
    TARGET_GRAPHQL_OPERATION_EXTENSIONS,
    Value::Object(Default::default()),
  );
  target
    .metadata
    .insert(METADATA_DOWNSTREAM_HTTP_REQUEST, downstream_req_value);

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

      if let Some(Value::Bytes(operation)) =
        target.value.remove(TARGET_GRAPHQL_OPERATION_KEY, false)
      {
        match String::from_utf8(operation.to_vec()) {
          Ok(operation_str) => {
            let raw_request = GraphQLRequest {
              operation: operation_str,
              extensions: None,
              variables: None,
              operation_name: None,
            };
            if let Err(e) = ParsedGraphQLRequest::create_and_parse(raw_request) {
              error!("Error parsing GraphQL request: {}", e);
              return;
            }
          }
          Err(e) => {
            error!("Error decoding bytes to string: {}", e);
            return;
          }
        }
      }

      if let Some(Value::Bytes(operation_name)) =
        target.value.remove(TARGET_GRAPHQL_OPERATION_NAME, false)
      {
        if let Ok(operation_name_str) = String::from_utf8(operation_name.to_vec()) {
          if let Some(request) = ctx.downstream_graphql_request.as_mut() {
            request.request.operation_name = Some(operation_name_str);
          }
        } else {
          error!("Failed to convert operation name to string");
          return;
        }
      }

      if let Some(Value::Object(variables)) = target
        .value
        .remove(TARGET_GRAPHQL_OPERATION_VARIABLES, false)
      {
        if !variables.is_empty() {
          match vrl_value_to_serde_value(&Value::Object(variables)) {
            Ok(serde_json::Value::Object(obj)) => {
              if let Some(request) = ctx.downstream_graphql_request.as_mut() {
                request.request.variables = Some(obj);
              }
            }
            Err(e) => {
              error!("Error converting VRL value to serde value: {}", e);
              return;
            }
            _ => {
              error!("Unexpected value type after conversion");
              return;
            }
          }
        }
      }

      if let Some(Value::Object(extensions)) = target
        .value
        .remove(TARGET_GRAPHQL_OPERATION_EXTENSIONS, false)
      {
        if !extensions.is_empty() {
          match vrl_value_to_serde_value(&Value::Object(extensions)) {
            Ok(serde_json::Value::Object(obj)) => {
              if let Some(request) = ctx.downstream_graphql_request.as_mut() {
                request.request.extensions = Some(obj);
              }
            }
            Err(e) => {
              error!("Error converting VRL value to serde value: {}", e);
            }
            _ => {
              error!("Unexpected value type after conversion");
            }
          }
        }
      }
    }
    Err(err) => {
      error!("vrl::on_downstream_http_request resolve error: {:?}", err);
      ctx.short_circuit(
        GraphQLResponse::new_error("vrl runtime error")
          .into_with_status_code(StatusCode::BAD_GATEWAY),
      );
    }
  }
}
