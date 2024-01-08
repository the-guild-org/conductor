use conductor_common::graphql::GraphQLError;
use conductor_common::graphql::ParsedGraphQLRequest;
use conductor_common::Definition;
use conductor_common::OperationDefinition;
use minitrace::Span;

use crate::otel_attrs::*;

// Based on https://opentelemetry.io/docs/specs/semconv/database/graphql/
#[inline]
pub fn create_graphql_span(request: &ParsedGraphQLRequest) -> Span {
  let excutable_op = request.executable_operation();

  let (op_type, op_name): (Option<&str>, Option<&String>) = match excutable_op {
    Some(Definition::Operation(op)) => match op {
      OperationDefinition::Query(o) => (Some("query"), o.name.as_ref()),
      OperationDefinition::SelectionSet(_) => (Some("query"), None),
      OperationDefinition::Mutation(o) => (Some("mutation"), o.name.as_ref()),
      OperationDefinition::Subscription(o) => (Some("subscription"), o.name.as_ref()),
    },
    _ => (None, None),
  };

  let otel_name = match (op_type, op_name) {
    (Some(op_type), Some(op_name)) => format!("{} {}", op_type, op_name),
    (Some(op_type), None) => op_type.to_string(),
    _ => "GraphQL Operation".to_string(),
  };

  let mut properties: Vec<(&str, String)> = Vec::new();
  properties.push((GRAPHQL_DOCUMENT, request.request.operation.to_string()));

  if let Some(op_type) = op_type {
    properties.push((GRAPHQL_OPERATION_TYPE, op_type.to_string()));
  }

  if let Some(op_name) = op_name {
    properties.push((GRAPHQL_OPERATION_NAME, op_name.to_string()));
  }

  Span::enter_with_local_parent(otel_name).with_properties(|| properties)
}

#[inline]
pub fn create_graphql_error_span_properties(
  errors: &Vec<GraphQLError>,
) -> impl IntoIterator<Item = (&'static str, String)> {
  let mut properties: Vec<(&str, String)> = Vec::new();

  if !errors.is_empty() {
    properties.push((GRAPHQL_ERROR_COUNT, errors.len().to_string()));
    properties.push((ERROR_TYPE, "graphql".to_string()));
    properties.push((OTEL_STATUS_CODE, "ERROR".to_string()));

    let errors_str = errors
      .iter()
      .map(|e| e.message.clone())
      .collect::<Vec<_>>()
      .join(", ");

    properties.push((ERROR_INDICATOR, "true".to_string()));
    properties.push((ERROR_MESSAGE, errors_str));
  }

  properties
}
