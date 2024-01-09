use crate::config::HttpGetPluginConfig;

use conductor_common::execute::RequestExecutionContext;
use conductor_common::{
  graphql::{
    ExtractGraphQLOperationError, GraphQLRequest, GraphQLResponse, ParsedGraphQLRequest,
    APPLICATION_GRAPHQL_JSON,
  },
  http::{
    extract_accept, extract_content_type, parse_query_string, ConductorHttpRequest, Method, Mime,
    StatusCode, APPLICATION_JSON, APPLICATION_WWW_FORM_URLENCODED,
  },
  json::parse_and_extract_json_map_value,
};

use conductor_common::plugin::{CreatablePlugin, Plugin, PluginError};

#[derive(Debug)]
pub struct HttpGetPlugin(HttpGetPluginConfig);

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for HttpGetPlugin {
  type Config = HttpGetPluginConfig;

  async fn create(config: Self::Config) -> Result<Box<dyn Plugin>, PluginError> {
    Ok(Box::new(Self(config)))
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for HttpGetPlugin {
  async fn on_downstream_http_request(&self, ctx: &mut RequestExecutionContext) {
    if ctx.downstream_http_request.method == Method::GET {
      let (_, accept, result) = extract_graphql_from_get_request(&ctx.downstream_http_request);

      match result {
        Ok(gql_request) => match ParsedGraphQLRequest::create_and_parse(gql_request) {
          Ok(parsed) => {
            ctx.downstream_graphql_request = Some(parsed);
          }
          Err(e) => {
            ctx.short_circuit(
              ExtractGraphQLOperationError::GraphQLParserError(e).into_response(accept),
            );
          }
        },
        Err(ExtractGraphQLOperationError::EmptyExtraction) => {
          // nothing to do here, maybe other plugins (like GraphiQL will take care of this one)
        }
        Err(e) => {
          ctx.short_circuit(e.into_response(accept));
        }
      }
    }
  }

  async fn on_downstream_graphql_request(&self, ctx: &mut RequestExecutionContext) {
    if ctx.downstream_http_request.method == Method::GET
      && (self.0.mutations.is_none() || self.0.mutations == Some(false))
    {
      if let Some(gql_req) = &ctx.downstream_graphql_request {
        if gql_req.is_running_mutation() {
          ctx.short_circuit(
            GraphQLResponse::new_error("mutations are not allowed over GET")
              .into_with_status_code(StatusCode::METHOD_NOT_ALLOWED),
          );
        }
      }
    }
  }
}

pub type ExtractionResult = (
  Option<Mime>,
  Option<Mime>,
  Result<GraphQLRequest, ExtractGraphQLOperationError>,
);

pub fn extract_graphql_from_get_request(
  downstream_request: &ConductorHttpRequest,
) -> ExtractionResult {
  let content_type = extract_content_type(&downstream_request.headers);
  let accept = extract_accept(&downstream_request.headers);

  if content_type == Some(APPLICATION_WWW_FORM_URLENCODED)
    || accept == Some(APPLICATION_JSON)
    || accept == Some(APPLICATION_GRAPHQL_JSON.parse::<Mime>().unwrap())
  {
    let params = parse_query_string(&downstream_request.query_string);

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
            operation: Some(operation.to_string()),
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
