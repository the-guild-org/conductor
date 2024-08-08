use std::{future::Future, pin::Pin, sync::Arc};

use conductor_common::{
  execute::RequestExecutionContext,
  graphql::{GraphQLResponse, ParsedGraphQLSchema},
  http::{ConductorHttpRequest, CONTENT_TYPE},
  plugin_manager::PluginManager,
};
use conductor_config::GraphQLSourceConfig;
use fastrace_reqwest::{traced_reqwest, TracedHttpClient};
use reqwest::{header::HeaderValue, Method, StatusCode};
use tracing::debug;

use crate::schema_awareness::SchemaAwareness;

use conductor_common::source::{GraphQLSourceInitError, SourceError, SourceRuntime};

#[derive(Debug)]
pub struct GraphQLSourceRuntime {
  pub fetcher: TracedHttpClient,
  pub config: GraphQLSourceConfig,
  pub identifier: String,
  pub schema_awareness: Option<SchemaAwareness>,
}

impl GraphQLSourceRuntime {
  pub async fn new(
    identifier: String,
    config: GraphQLSourceConfig,
  ) -> Result<Self, GraphQLSourceInitError> {
    tracing::info!(
      "Initializing source '{}' of type 'graphql' with config: {:?}",
      identifier,
      config
    );

    let client = wasm_polyfills::create_http_client()
      .build()
      .map_err(|source| GraphQLSourceInitError::FetcherError { source })?;

    let fetcher = traced_reqwest(client);
    let schema_awareness = match config.schema_awareness.as_ref() {
      Some(c) => Some(
        SchemaAwareness::new(identifier.clone(), c.to_owned(), |_, _| Ok(()))
          .await
          .map_err(|source| GraphQLSourceInitError::SourceInitFailed {
            source: source.into(),
          })?,
      ),
      None => None,
    };

    Ok(Self {
      schema_awareness,
      identifier,
      fetcher,
      config,
    })
  }
}

impl SourceRuntime for GraphQLSourceRuntime {
  fn name(&self) -> &str {
    &self.identifier
  }

  fn sdl(&self) -> Option<Arc<String>> {
    if let Some(schema_awareness) = &self.schema_awareness {
      return schema_awareness.raw();
    }

    None
  }

  fn schema(&self) -> Option<Arc<ParsedGraphQLSchema>> {
    if let Some(schema_awareness) = &self.schema_awareness {
      return schema_awareness.schema();
    }

    None
  }

  fn execute<'a>(
    &'a self,
    plugin_manager: Arc<Box<dyn PluginManager>>,
    request_context: &'a mut RequestExecutionContext,
  ) -> Pin<Box<(dyn Future<Output = Result<GraphQLResponse, SourceError>> + 'a)>> {
    Box::pin(wasm_polyfills::call_async(async move {
      let fetcher = &self.fetcher;
      let endpoint = &self.config.endpoint;

      let source_req = match request_context.downstream_graphql_request.as_mut() {
        Some(req) => &mut req.request,
        None => {
          return Ok(GraphQLResponse::new_error(
            "source request isn't available at execution context!",
          ))
        }
      };

      plugin_manager.on_upstream_graphql_request(source_req).await;

      // TODO: improve this by implementing https://github.com/the-guild-org/conductor-t2/issues/205
      let mut conductor_http_request = ConductorHttpRequest {
        body: source_req.into(),
        uri: endpoint.to_string(),
        query_string: "".to_string(),
        method: Method::POST,
        headers: Default::default(),
      };

      conductor_http_request
        .headers
        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

      plugin_manager
        .on_upstream_http_request(request_context, &mut conductor_http_request)
        .await;

      if request_context.is_short_circuit() {
        return Err(SourceError::ShortCircuit);
      }

      debug!(
        "dispatching upstream http request from the following input: {:?}",
        conductor_http_request
      );

      let upstream_req = fetcher
        .request(conductor_http_request.method, conductor_http_request.uri)
        .headers(conductor_http_request.headers)
        .body(conductor_http_request.body);

      let upstream_response = upstream_req.send().await;

      plugin_manager
        .on_upstream_http_response(request_context, &upstream_response)
        .await;

      match upstream_response {
        Ok(res) => match res.status() {
          StatusCode::OK => {
            let body = match res.bytes().await {
              Ok(body) => body,
              Err(e) => return Ok(GraphQLResponse::new_error(&e.to_string())),
            };

            // DOTAN: Should we use the improved JSON parser here?
            let response = match serde_json::from_slice::<GraphQLResponse>(&body) {
              Ok(response) => response,
              Err(e) => {
                return Ok(GraphQLResponse::new_error(&format!(
                  "Failed to build json response {}",
                  e
                )))
              }
            };

            Ok(response)
          }
          code => Err(SourceError::UnexpectedHTTPStatusError(code)),
        },
        Err(e) => Err(SourceError::NetworkError(e)),
      }
    }))
  }
}
