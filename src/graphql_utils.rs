use axum::response::{IntoResponse, Response};
use graphql_parser::{
    parse_query,
    query::{Definition, Document, OperationDefinition, ParseError},
};
use http::{header::CONTENT_TYPE, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::http_utils::ExtractGraphQLOperationError;
pub const APPLICATION_GRAPHQL_JSON: &str = "application/graphql-response+json";

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

/// An error with a message and optional extensions.
#[derive(Debug, Clone, Serialize)]
pub struct GraphQLError {
    /// The error message.
    pub message: String,
    /// Extensions to the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Map<String, Value>>,
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
    pub fn create_and_parse(
        raw_request: GraphQLRequest,
    ) -> Result<Self, ExtractGraphQLOperationError> {
        parse_graphql_operation(&raw_request.operation)
            .map(|parsed_operation| ParsedGraphQLRequest {
                request: raw_request,
                parsed_operation,
            })
            .map_err(ExtractGraphQLOperationError::GraphQLParserError)
    }

    pub fn is_mutation(&self) -> bool {
        for definition in &self.parsed_operation.definitions {
            if let Definition::Operation(OperationDefinition::Mutation(_)) = definition {
                return true;
            }
        }

        false
    }
}

#[derive(Serialize, Debug)]
pub struct GraphQLResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<GraphQLError>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Value>,
}

impl GraphQLResponse {
    pub fn new_error(error: &str) -> Self {
        GraphQLResponse {
            data: None,
            errors: Some(vec![GraphQLError::new(error)]),
            extensions: None,
        }
    }

    pub fn into_empty_response(self, status_code: StatusCode) -> Response {
        (status_code).into_response()
    }

    pub fn into_response(self, status_code: StatusCode) -> Response {
        let body = serde_json::to_string(&self).unwrap();

        Response::builder()
            .status(status_code)
            .header(CONTENT_TYPE, "application/json")
            .body(axum::body::boxed(body))
            .unwrap()
    }
}

pub fn parse_graphql_operation(operation_str: &str) -> Result<ParsedGraphQLDocument, ParseError> {
    parse_query::<String>(operation_str).map(|v| v.into_static())
}
