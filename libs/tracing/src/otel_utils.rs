use conductor_common::graphql::ParsedGraphQLRequest;
use conductor_common::Definition;
use conductor_common::OperationDefinition;
use tracing::Span;

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

  tracing::info_span!(
    "graphql_execute",
    "graphql.operation.type" = op_type,
    "graphql.operation.name" = op_name,
    "graphql.document" = request.request.operation,
    "graphql.error.count" = tracing::field::Empty,
    "error.type" = tracing::field::Empty,
    "error.message" = tracing::field::Empty,
    "otel.name" = otel_name
  )
}
