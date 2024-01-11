use std::{future::Future, pin::Pin};

use conductor_common::{
  execute::RequestExecutionContext,
  graphql::GraphQLResponse,
  http::{ConductorHttpRequest, CONTENT_TYPE},
};
use conductor_config::GraphQLSourceConfig;
use reqwest::{header::HeaderValue, Client, Method, StatusCode};
use tracing::debug;

use crate::gateway::ConductorGatewayRouteData;

use super::runtime::{SourceError, SourceRuntime};

#[derive(Debug)]
pub struct GraphQLSourceRuntime {
  pub fetcher: Client,
  pub config: GraphQLSourceConfig,
}

impl GraphQLSourceRuntime {
  pub fn new(config: GraphQLSourceConfig) -> Self {
    let fetcher = wasm_polyfills::create_http_client()
      .build()
      .unwrap_or_else(|_| {
        // @expected: without a fetcher, there's no executor, without an executor, there's no gateway.
        panic!(
          "Failed while initializing the executor's fetcher for GraphQL Source \"{}\"",
          config.endpoint
        )
      });

    Self { fetcher, config }
  }
}

impl SourceRuntime for GraphQLSourceRuntime {
  // #[tracing::instrument(
  //   skip(self, route_data, request_context),
  //   name = "GraphQLSourceRuntime::execute"
  // )]
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
        "going to send upstream request from the following input: {:?}",
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

            // DOTAN: Yassin, should we use the improved JSON parser here?
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
