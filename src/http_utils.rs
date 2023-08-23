use graphql_parser::query::ParseError;
use http::{
    header::{ACCEPT, CONTENT_TYPE},
    HeaderMap, StatusCode,
};
use hyper::body::to_bytes;
use mime::Mime;
use mime::{APPLICATION_JSON, APPLICATION_WWW_FORM_URLENCODED};
use serde::de::Error as DeError;
use serde_json::{from_slice, from_str, Error as SerdeError, Map, Value};
use std::collections::HashMap;

use crate::{
    graphql_utils::{GraphQLRequest, GraphQLResponse, APPLICATION_GRAPHQL_JSON},
    plugins::flow_context::FlowContext,
};

pub fn extract_content_type(headers_map: &HeaderMap) -> Option<Mime> {
    let content_type = headers_map
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    content_type.and_then(|content_type| content_type.parse().ok())
}

pub fn extract_accept(headers_map: &HeaderMap) -> Option<Mime> {
    let content_type = headers_map
        .get(ACCEPT)
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string);

    content_type.and_then(|content_type| content_type.parse().ok())
}

#[derive(Debug)]
pub enum ExtractGraphQLOperationError {
    MissingQueryParameter,
    InvalidContentTypeHeader,
    InvalidBodyJsonFormat(SerdeError),
    InvalidVariablesJsonFormat(SerdeError),
    InvalidExtensionsJsonFormat(SerdeError),
    EmptyExtraction,
    FailedToReadRequestBody(hyper::Error),
    GraphQLParserError(ParseError),
}

impl ExtractGraphQLOperationError {
    pub fn into_response(self, accept: Option<Mime>) -> axum::response::Response {
        let gql_response = GraphQLResponse::new_error(&match &self {
            ExtractGraphQLOperationError::MissingQueryParameter => {
                "Missing GraphQL query parameter".to_string()
            }
            ExtractGraphQLOperationError::InvalidVariablesJsonFormat(e) => {
                format!("Invalid GraphQL variables JSON format: {}", e)
            }
            ExtractGraphQLOperationError::InvalidBodyJsonFormat(e) => {
                format!("Invalid GraphQL request JSON format: {}", e)
            }
            ExtractGraphQLOperationError::InvalidExtensionsJsonFormat(e) => {
                format!("Invalid GraphQL extensions JSON format: {}", e)
            }
            ExtractGraphQLOperationError::InvalidContentTypeHeader => {
                "Invalid Content-Type header value, expected application/json".to_string()
            }
            ExtractGraphQLOperationError::EmptyExtraction => {
                "Failed to location a GraphQL query in request".to_string()
            }
            ExtractGraphQLOperationError::FailedToReadRequestBody(e) => {
                format!("Failed to read response body: {}", e)
            }
            ExtractGraphQLOperationError::GraphQLParserError(e) => e.to_string(),
        });

        match accept {
            None => gql_response.into_response(StatusCode::OK),
            Some(accept_header) => {
                if let ExtractGraphQLOperationError::GraphQLParserError(_) = &self {
                    if accept_header == APPLICATION_JSON {
                        return gql_response.into_empty_response(StatusCode::OK);
                    }
                }

                gql_response.into_empty_response(StatusCode::BAD_REQUEST)
            }
        }
    }
}

pub async fn extract_graphql_from_post_request<'a>(
    flow_ctx: &mut FlowContext<'a>,
) -> ExtractionResult {
    // Extract the content-type and default to application/json when it's not set
    // see https://graphql.github.io/graphql-over-http/draft/#sec-POST
    let headers = flow_ctx.downstream_http_request.headers();
    let content_type = extract_content_type(headers).unwrap_or(APPLICATION_JSON);
    let accept = extract_accept(headers);

    if content_type.type_() != mime::APPLICATION_JSON.type_() {
        return (
            Some(content_type),
            accept,
            Err(ExtractGraphQLOperationError::InvalidContentTypeHeader),
        );
    }

    let body_bytes = to_bytes(flow_ctx.downstream_http_request.body_mut()).await;

    match body_bytes {
        Ok(bytes) => (
            Some(content_type),
            accept,
            from_slice(bytes.as_ref()).map_err(ExtractGraphQLOperationError::InvalidBodyJsonFormat),
        ),
        Err(e) => (
            Some(content_type),
            accept,
            Err(ExtractGraphQLOperationError::FailedToReadRequestBody(e)),
        ),
    }
}

pub fn parse_and_extract_json_map_value(value: &str) -> Result<Map<String, Value>, SerdeError> {
    let parsed_json = from_str::<Value>(value);

    match parsed_json {
        Ok(Value::Object(v)) => Ok(v),
        Ok(_) => Err(DeError::custom("expected object")),
        Err(e) => Err(e),
    }
}

pub type ExtractionResult = (
    Option<Mime>,
    Option<Mime>,
    Result<GraphQLRequest, ExtractGraphQLOperationError>,
);

pub fn extract_graphql_from_get_request(flow_ctx: &mut FlowContext) -> ExtractionResult {
    let headers = flow_ctx.downstream_http_request.headers();
    let content_type = extract_content_type(headers);
    let accept = extract_accept(headers);

    if content_type == Some(APPLICATION_WWW_FORM_URLENCODED)
        || accept == Some(APPLICATION_JSON)
        || accept == Some(APPLICATION_GRAPHQL_JSON.parse::<Mime>().unwrap())
    {
        let params: HashMap<String, String> = flow_ctx
            .downstream_http_request
            .uri()
            .query()
            .map(|v| {
                url::form_urlencoded::parse(v.as_bytes())
                    .into_owned()
                    .collect()
            })
            .unwrap_or_else(HashMap::new);

        match params.get("query") {
            Some(operation) => {
                let operation_name = params.get("operationName");

                let variables = match params.get("variables") {
                    Some(v) => match parse_and_extract_json_map_value(v) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            return (
                                content_type,
                                accept,
                                Err(ExtractGraphQLOperationError::InvalidVariablesJsonFormat(e)),
                            )
                        }
                    },
                    None => None,
                };
                let extensions = match params.get("extensions") {
                    Some(v) => match parse_and_extract_json_map_value(v) {
                        Ok(v) => Some(v),
                        Err(e) => {
                            return (
                                content_type,
                                accept,
                                Err(ExtractGraphQLOperationError::InvalidExtensionsJsonFormat(e)),
                            )
                        }
                    },
                    None => None,
                };

                return (
                    content_type,
                    accept,
                    Ok(GraphQLRequest {
                        operation: operation.to_string(),
                        operation_name: operation_name.map(ToString::to_string),
                        variables,
                        extensions,
                    }),
                );
            }
            None => {
                return (
                    content_type,
                    accept,
                    Err(ExtractGraphQLOperationError::MissingQueryParameter),
                )
            }
        }
    }

    if content_type.is_none() {
        return (
            content_type,
            accept,
            Err(ExtractGraphQLOperationError::EmptyExtraction),
        );
    }

    (
        content_type,
        accept,
        Err(ExtractGraphQLOperationError::InvalidContentTypeHeader),
    )
}
