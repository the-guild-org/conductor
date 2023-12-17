use bytes::Bytes;
use graphql_parser::{
    parse_query,
    query::{Definition, Document, OperationDefinition, ParseError},
};
use mime::{Mime, APPLICATION_JSON};
use serde::{Deserialize, Serialize};
use serde_json::{Error as SerdeError, Map, Value};

use crate::http::{
    extract_accept, extract_content_type, ConductorHttpRequest, ConductorHttpResponse, StatusCode,
};

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

#[derive(thiserror::Error, Debug)]
pub enum ExtractGraphQLOperationError {
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
        serde_json::to_vec(&request).unwrap().into()
    }
}

impl From<GraphQLRequest> for Bytes {
    fn from(value: GraphQLRequest) -> Self {
        serde_json::to_vec(&value).unwrap().into()
    }
}

impl From<&GraphQLRequest> for Bytes {
    fn from(request: &GraphQLRequest) -> Self {
        serde_json::to_vec(&request).unwrap().into()
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
    #[tracing::instrument(
        level = "debug",
        name = "ParsedGraphQLRequest::parse_graphql_operation"
    )]
    pub fn create_and_parse(raw_request: GraphQLRequest) -> Result<Self, ParseError> {
        parse_graphql_operation(&raw_request.operation).map(|parsed_operation| {
            ParsedGraphQLRequest {
                request: raw_request,
                parsed_operation,
            }
        })
    }

    #[tracing::instrument(level = "trace", name = "ParsedGraphQLRequest::is_running_mutation")]
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
}

impl GraphQLResponse {
    pub fn new_error(error: &str) -> Self {
        GraphQLResponse {
            data: None,
            errors: Some(vec![GraphQLError::new(error)]),
            extensions: None,
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
        serde_json::to_vec(&response).unwrap().into()
    }
}

impl From<GraphQLResponse> for ConductorHttpResponse {
    fn from(response: GraphQLResponse) -> Self {
        ConductorHttpResponse {
            body: response.into(),
            status: StatusCode::OK,
            headers: Default::default(),
        }
    }
}

pub fn parse_graphql_operation(operation_str: &str) -> Result<ParsedGraphQLDocument, ParseError> {
    parse_query::<String>(operation_str).map(|v| v.into_static())
}
