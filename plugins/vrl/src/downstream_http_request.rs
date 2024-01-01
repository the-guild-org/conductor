use conductor_common::{
  graphql::{GraphQLRequest, GraphQLResponse, ParsedGraphQLRequest},
  http::StatusCode,
  vrl_utils::vrl_value_to_serde_value,
};
use tracing::{error, warn};
use vrl::{
  compiler::{Context, Program, TargetValue, TimeZone},
  value,
  value::{Secrets, Value},
};

use conductor_common::execute::RequestExecutionContext;

use super::{utils::conductor_request_to_value, vrl_functions::ShortCircuitFn};

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
          GraphQLResponse::new_error(&String::from_utf8(message.to_vec()).unwrap())
            .into_with_status_code(StatusCode::from_u16(error_code as u16).unwrap()),
        );

        return;
      }

      if let Some(Value::Bytes(operation)) =
        target.value.remove(TARGET_GRAPHQL_OPERATION_KEY, false)
      {
        let raw_request = GraphQLRequest {
          operation: String::from_utf8(operation.into()).unwrap(),
          extensions: None,
          variables: None,
          operation_name: None,
        };

        ctx.downstream_graphql_request =
          Some(ParsedGraphQLRequest::create_and_parse(raw_request).unwrap());
      }

      if let Some(Value::Bytes(operation_name)) =
        target.value.remove(TARGET_GRAPHQL_OPERATION_NAME, false)
      {
        if ctx.downstream_graphql_request.is_none() {
          warn!("vrl::on_downstream_http_request: operation_name is set, but operation is not set, ignoring.");
        } else {
          ctx
            .downstream_graphql_request
            .as_mut()
            .unwrap()
            .request
            .operation_name = Some(String::from_utf8(operation_name.into()).unwrap())
        }
      }

      if let Some(Value::Object(variables)) = target
        .value
        .remove(TARGET_GRAPHQL_OPERATION_VARIABLES, false)
      {
        if variables.keys().len() > 0 {
          if let serde_json::Value::Object(obj) =
            vrl_value_to_serde_value(&Value::Object(variables))
          {
            ctx
              .downstream_graphql_request
              .as_mut()
              .unwrap()
              .request
              .variables = Some(obj);
          }
        }
      }

      if let Some(Value::Object(extensions)) = target
        .value
        .remove(TARGET_GRAPHQL_OPERATION_EXTENSIONS, false)
      {
        if extensions.keys().len() > 0 {
          if let serde_json::Value::Object(obj) =
            vrl_value_to_serde_value(&Value::Object(extensions))
          {
            ctx
              .downstream_graphql_request
              .as_mut()
              .unwrap()
              .request
              .extensions = Some(obj);
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
