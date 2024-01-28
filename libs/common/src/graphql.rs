use std::fmt::{Display, Formatter};

use bytes::Bytes;
use graphql_parser::{
  parse_query,
  query::{Definition, Document, OperationDefinition, ParseError},
};
use mime::{Mime, APPLICATION_JSON};
use minitrace::trace;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::{Error as SerdeError, Map, Value};

use crate::http::{
  extract_accept, extract_content_type, ConductorHttpRequest, ConductorHttpResponse, StatusCode,
};

pub const APPLICATION_GRAPHQL_JSON: &str = "application/graphql-response+json";
pub static APPLICATION_GRAPHQL_JSON_MIME: Lazy<Mime> = Lazy::new(|| {
  APPLICATION_GRAPHQL_JSON
    .parse::<Mime>()
    // @expected: we're parsing a statically defined constant, we know it works ;)
    .unwrap()
});

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GraphQLRequest {
  // The GraphQL operation, as string
  #[serde(rename = "query")]
  pub operation: String,
  // The operation name, if specified
  #[serde(rename = "operationName")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub operation_name: Option<String>,
  // GraphQL operation variables, in JSON format
  pub variables: Option<Map<String, Value>>,
  // GraphQL execution extensions, in JSON format
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extensions: Option<Map<String, Value>>,
}

#[cfg(feature = "test_utils")]
impl Default for GraphQLRequest {
  fn default() -> Self {
    GraphQLRequest {
      operation: "query { __typename }".to_string(),
      operation_name: None,
      variables: None,
      extensions: None,
    }
  }
}

impl Display for GraphQLRequest {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      serde_json::to_string(self)
        .unwrap_or_else(|e| ExtractGraphQLOperationError::SerializationError(e).to_string())
    )
  }
}

#[derive(thiserror::Error, Debug)]
pub enum ExtractGraphQLOperationError {
  #[error("invalid url query parameter")]
  InvalidQueryParameterEncoding,
  #[error("missing query parameter")]
  MissingQueryParameter,
  #[error("invalid content-type header")]
  InvalidContentTypeHeader,
  #[error("invalid body json format")]
  InvalidBodyJsonFormat(SerdeError),
  #[error("invalid variables json format")]
  InvalidVariablesJsonFormat(SerdeError),
  #[error("invalid extensions json format")]
  InvalidExtensionsJsonFormat(SerdeError),
  #[error("failed to create response body")]
  FailedToCreateResponseBody,
  #[error("failed to read request body")]
  FailedToReadRequestBody,
  #[error("failed to parse GraphQL operation")]
  GraphQLParserError(ParseError),
  #[error("failed to locate any GraphQL operation in request")]
  EmptyExtraction,
  #[error("serialization error")]
  SerializationError(SerdeError),
}

impl ExtractGraphQLOperationError {
  pub fn into_response(&self, accept: Option<Mime>) -> ConductorHttpResponse {
    match accept {
      None => ConductorHttpResponse {
        body: GraphQLResponse::new_error(self.to_string().as_str()).into(),
        status: StatusCode::OK,
        headers: Default::default(),
      },
      Some(accept_header) => {
        if let ExtractGraphQLOperationError::GraphQLParserError(_) = &self {
          if accept_header == APPLICATION_JSON {
            return ConductorHttpResponse {
              body: GraphQLResponse::new_error(self.to_string().as_str()).into(),
              status: StatusCode::OK,
              headers: Default::default(),
            };
          }
        }

        ConductorHttpResponse {
          body: GraphQLResponse::new_error(self.to_string().as_str()).into(),
          status: StatusCode::BAD_REQUEST,
          headers: Default::default(),
        }
      }
    }
  }
}

impl GraphQLRequest {
  pub fn new_from_http_post(
    http_request: &ConductorHttpRequest,
  ) -> (
    Option<Mime>,
    Option<Mime>,
    Result<GraphQLRequest, ExtractGraphQLOperationError>,
  ) {
    // Extract the content-type and default to application/json when it's not set
    // see https://graphql.github.io/graphql-over-http/draft/#sec-POST
    let content_type = extract_content_type(&http_request.headers).unwrap_or(APPLICATION_JSON);
    let accept = extract_accept(&http_request.headers);

    if content_type.type_() != mime::APPLICATION_JSON.type_() {
      return (
        Some(content_type),
        accept,
        Err(ExtractGraphQLOperationError::InvalidContentTypeHeader),
      );
    }

    match http_request.json_body::<GraphQLRequest>() {
      Ok(body) => (Some(content_type), accept, Ok(body)),
      Err(e) => (
        Some(content_type),
        accept,
        Err(ExtractGraphQLOperationError::InvalidBodyJsonFormat(e)),
      ),
    }
  }
}

impl From<&mut GraphQLRequest> for Bytes {
  fn from(request: &mut GraphQLRequest) -> Self {
    serde_json::to_vec(&request)
      .unwrap_or_else(|e| {
        ExtractGraphQLOperationError::SerializationError(e)
          .to_string()
          .into_bytes()
      })
      .into()
  }
}

impl From<GraphQLRequest> for Bytes {
  fn from(value: GraphQLRequest) -> Self {
    serde_json::to_vec(&value)
      .unwrap_or_else(|e| {
        ExtractGraphQLOperationError::SerializationError(e)
          .to_string()
          .into_bytes()
      })
      .into()
  }
}

impl From<&GraphQLRequest> for Bytes {
  fn from(request: &GraphQLRequest) -> Self {
    serde_json::to_vec(&request)
      .unwrap_or_else(|e| {
        ExtractGraphQLOperationError::SerializationError(e)
          .to_string()
          .into_bytes()
      })
      .into()
  }
}

/// An error with a message and optional extensions.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphQLError {
  /// The error message.
  pub message: String,
  /// Extensions to the error.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extensions: Option<Map<String, Value>>,
}

impl std::fmt::Display for GraphQLError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}

impl GraphQLError {
  pub fn new(message: &str) -> Self {
    GraphQLError {
      message: message.to_string(),
      extensions: None,
    }
  }
}

pub type ParsedGraphQLDocument = Document<'static, String>;

#[derive(Debug)]
pub struct ParsedGraphQLRequest {
  pub request: GraphQLRequest,
  pub parsed_operation: ParsedGraphQLDocument,
}

impl ParsedGraphQLRequest {
  #[trace(name = "graphql_parse")]
  pub fn create_and_parse(raw_request: GraphQLRequest) -> Result<Self, ParseError> {
    parse_graphql_operation(&raw_request.operation).map(|parsed_operation| ParsedGraphQLRequest {
      request: raw_request,
      parsed_operation,
    })
  }

  pub fn executable_operation(&self) -> Option<&Definition<'static, String>> {
    match &self.request.operation_name {
      Some(op_name) => self.parsed_operation.definitions.iter().find(|v| {
        if let Definition::Operation(op) = v {
          let name: &Option<String> = match op {
            OperationDefinition::SelectionSet(_) => &None,
            OperationDefinition::Query(query) => &query.name,
            OperationDefinition::Mutation(mutation) => &mutation.name,
            OperationDefinition::Subscription(subscription) => &subscription.name,
          };

          if let Some(actual_name) = name {
            return actual_name == op_name;
          }
        }

        false
      }),
      _ => self.parsed_operation.definitions.iter().find(|v| {
        if let Definition::Operation(_) = v {
          return true;
        }

        false
      }),
    }
  }

  pub fn is_introspection_query(&self) -> bool {
    let operation_to_execute = self.executable_operation();
    let root_level_selections = match operation_to_execute {
      Some(Definition::Operation(OperationDefinition::SelectionSet(s))) => Some(s),
      Some(Definition::Operation(OperationDefinition::Query(q))) => Some(&q.selection_set),
      _ => None,
    };

    if let Some(selections) = root_level_selections {
      let all_typename = selections.items.iter().all(|v| {
        // TODO: Should we handle Fragments here as well?
        if let graphql_parser::query::Selection::Field(field) = v {
          return field.name == "__typename";
        }

        false
      });

      if all_typename {
        return true;
      }

      let has_some_introspection_fields = selections.items.iter().any(|v| {
        if let graphql_parser::query::Selection::Field(field) = v {
          return field.name == "__schema" || field.name == "__type";
        }

        false
      });

      if has_some_introspection_fields {
        return true;
      }
    }

    false
  }

  #[tracing::instrument(
    level = "trace",
    name = "ParsedGraphQLRequest::is_running_mutation",
    skip_all
  )]
  pub fn is_running_mutation(&self) -> bool {
    if let Some(operation_name) = &self.request.operation_name {
      for definition in &self.parsed_operation.definitions {
        if let Definition::Operation(OperationDefinition::Mutation(mutation)) = definition {
          if let Some(mutation_name) = &mutation.name {
            if *mutation_name == *operation_name {
              return true;
            }
          }
        }
      }
    } else {
      for definition in &self.parsed_operation.definitions {
        if let Definition::Operation(OperationDefinition::Mutation(_)) = definition {
          return true;
        }
      }
    }

    false
  }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GraphQLResponse {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<Value>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub errors: Option<Vec<GraphQLError>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub extensions: Option<Value>,

  #[serde(skip)]
  downstream_http_code: Option<StatusCode>,
}

impl GraphQLResponse {
  pub fn append_extensions(&mut self, extensions: Map<String, Value>) {
    if let Some(existing_extensions) = &mut self.extensions {
      if let Value::Object(existing_extensions_map) = existing_extensions {
        existing_extensions_map.extend(extensions);
      }
    } else {
      self.extensions = Some(Value::Object(extensions));
    }
  }

  pub fn new_error(error: &str) -> Self {
    GraphQLResponse {
      data: None,
      errors: Some(vec![GraphQLError::new(error)]),
      extensions: None,
      downstream_http_code: None,
    }
  }

  pub fn new_error_with_code(error: &str, status_code: StatusCode) -> Self {
    GraphQLResponse {
      data: None,
      errors: Some(vec![GraphQLError::new(error)]),
      extensions: None,
      downstream_http_code: Some(status_code),
    }
  }

  pub fn into_with_status_code(self, code: StatusCode) -> ConductorHttpResponse {
    ConductorHttpResponse {
      body: self.into(),
      status: code,
      headers: Default::default(),
    }
  }
}

impl From<GraphQLResponse> for Bytes {
  fn from(response: GraphQLResponse) -> Self {
    serde_json::to_vec(&response)
      .unwrap_or_else(|e| {
        ExtractGraphQLOperationError::SerializationError(e)
          .to_string()
          .into_bytes()
      })
      .into()
  }
}

impl From<GraphQLResponse> for ConductorHttpResponse {
  fn from(response: GraphQLResponse) -> Self {
    let status = response.downstream_http_code.unwrap_or(StatusCode::OK);

    ConductorHttpResponse {
      body: response.into(),
      status,
      headers: Default::default(),
    }
  }
}

pub fn parse_graphql_operation(operation_str: &str) -> Result<ParsedGraphQLDocument, ParseError> {
  parse_query::<String>(operation_str).map(|v| v.into_static())
}
