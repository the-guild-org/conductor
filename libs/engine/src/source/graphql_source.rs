use std::{future::Future, pin::Pin};

use conductor_common::{
  execute::RequestExecutionContext,
  graphql::GraphQLResponse,
  http::{ConductorHttpRequest, CONTENT_TYPE},
};
use conductor_config::GraphQLSourceConfig;
use minitrace_reqwest::{traced_reqwest, TracedHttpClient};
use reqwest::{header::HeaderValue, Method, StatusCode};
use tracing::debug;

use crate::gateway::ConductorGatewayRouteData;

use super::runtime::{SourceError, SourceRuntime};

#[derive(Debug)]
pub struct GraphQLSourceRuntime {
  pub fetcher: TracedHttpClient,
  pub config: GraphQLSourceConfig,
  pub identifier: String,
}

impl GraphQLSourceRuntime {
  pub fn new(identifier: String, config: GraphQLSourceConfig) -> Self {
    let client = wasm_polyfills::create_http_client()
      .build()
      .unwrap_or_else(|_| {
        // @expected: without a fetcher, there's no executor, without an executor, there's no gateway.
        panic!(
          "Failed while initializing the executor's fetcher for GraphQL Source \"{}\"",
          config.endpoint
        )
      });

    let fetcher = traced_reqwest(client);

    Self {
      identifier,
      fetcher,
      config,
    }
  }
}

impl SourceRuntime for GraphQLSourceRuntime {
  fn name(&self) -> &str {
    &self.identifier
  }

  fn execute<'a>(
    &'a self,
    route_data: &'a ConductorGatewayRouteData,
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

      route_data
        .plugin_manager
        .on_upstream_graphql_request(source_req)
        .await;

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

      route_data
        .plugin_manager
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

      route_data
        .plugin_manager
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
