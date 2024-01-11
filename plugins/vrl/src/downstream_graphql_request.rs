use conductor_common::{
  graphql::{GraphQLRequest, GraphQLResponse, ParsedGraphQLRequest},
  http::StatusCode,
  vrl_utils::vrl_value_to_serde_value,
};
use tracing::error;
use vrl::{
  compiler::{Context, Program, TargetValue, TimeZone},
  value,
  value::{Secrets, Value},
};

use conductor_common::execute::RequestExecutionContext;

use super::{utils::conductor_graphql_request_to_value, vrl_functions::ShortCircuitFn};

static METADATA_GRAPHQL_OPERATION_INFO: &str = "downstream_graphql_req";
static TARGET_GRAPHQL_OPERATION_KEY: &str = "graphql.operation";
static TARGET_GRAPHQL_OPERATION_NAME: &str = "graphql.operation_name";
static TARGET_GRAPHQL_OPERATION_VARIABLES: &str = "graphql.variables";
static TARGET_GRAPHQL_OPERATION_EXTENSIONS: &str = "graphql.extensions";

pub fn vrl_downstream_graphql_request(program: &Program, ctx: &mut RequestExecutionContext) {
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

  if let Some(gql_req) = ctx.downstream_graphql_request.as_ref() {
    match conductor_graphql_request_to_value(&gql_req.request) {
      Ok(value) => {
        target
          .metadata
          .insert(METADATA_GRAPHQL_OPERATION_INFO, value);
      }
      Err(e) => {
        return ctx.short_circuit(GraphQLResponse::new_error(&e.to_string()).into());
      }
    }
  } else {
    return ctx.short_circuit(GraphQLResponse::new_error("GraphQL Request is missing!").into());
  }

  match program.resolve(&mut Context::new(
    &mut target,
    ctx.vrl_shared_state(),
    &TimeZone::default(),
  )) {
    Ok(ret) => {
      if let Some((error_code, message)) = ShortCircuitFn::check_short_circuit(&ret) {
        return ctx.short_circuit(
          GraphQLResponse::new_error(&String::from_utf8_lossy(&message)).into_with_status_code(
            StatusCode::from_u16(error_code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
          ),
        );
      }

      if let Some(Value::Bytes(operation)) =
        target.value.remove(TARGET_GRAPHQL_OPERATION_KEY, false)
      {
        match String::from_utf8(operation.to_vec()) {
          Ok(operation_str) => {
            match ParsedGraphQLRequest::create_and_parse(GraphQLRequest {
              operation: operation_str,
              extensions: None,
              variables: None,
              operation_name: None,
            }) {
              Ok(request) => ctx.downstream_graphql_request = Some(request),
              Err(e) => {
                ctx.short_circuit(GraphQLResponse::new_error(&e.to_string()).into());
                return;
              }
            };
          }
          Err(e) => {
            ctx.short_circuit(GraphQLResponse::new_error(&e.to_string()).into());
            return;
          }
        }
      }

      if let Some(Value::Bytes(operation_name)) =
        target.value.remove(TARGET_GRAPHQL_OPERATION_NAME, false)
      {
        match String::from_utf8(operation_name.to_vec()) {
          Ok(operation_name_str) => {
            if let Some(downstream_req) = ctx.downstream_graphql_request.as_mut() {
              downstream_req.request.operation_name = Some(operation_name_str);
            }
          }
          Err(e) => {
            return ctx.short_circuit(GraphQLResponse::new_error(&e.to_string()).into());
          }
        }
      }

      if let Some(Value::Object(variables)) = target
        .value
        .remove(TARGET_GRAPHQL_OPERATION_VARIABLES, false)
      {
        if !variables.is_empty() {
          match vrl_value_to_serde_value(&Value::Object(variables)) {
            Ok(serde_json::Value::Object(obj)) => {
              if let Some(downstream_req) = ctx.downstream_graphql_request.as_mut() {
                downstream_req.request.variables = Some(obj);
              }
            }
            Err(e) => {
              ctx.short_circuit(GraphQLResponse::new_error(&e.to_string()).into());
              return;
            }
            _ => {
              ctx.short_circuit(
                GraphQLResponse::new_error("Unexpected value type after conversion").into(),
              );
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
              if let Some(downstream_req) = ctx.downstream_graphql_request.as_mut() {
                downstream_req.request.extensions = Some(obj);
              }
            }
            Err(e) => {
              ctx.short_circuit(GraphQLResponse::new_error(&e.to_string()).into());
              return;
            }
            _ => {
              ctx.short_circuit(
                GraphQLResponse::new_error("Unexpected value type after conversion").into(),
              );
              return;
            }
          }
        }
      }
    }
    Err(err) => {
      error!(
        "vrl::vrl_downstream_graphql_request resolve error: {:?}",
        err
      );
      return ctx.short_circuit(
        GraphQLResponse::new_error("vrl runtime error")
          .into_with_status_code(StatusCode::BAD_GATEWAY),
      );
    }
  }
}
