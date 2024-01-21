use super::runtime::{SourceError, SourceRuntime};
use crate::gateway::ConductorGatewayRouteData;
use base64::{engine, Engine};
use conductor_common::execute::RequestExecutionContext;
use conductor_common::graphql::GraphQLResponse;
use conductor_config::{FederationSourceConfig, SupergraphSourceConfig};
use federation_query_planner::execute_federation;
use federation_query_planner::supergraph::{parse_supergraph, Supergraph};
use minitrace_reqwest::{traced_reqwest, TracedHttpClient};
use std::collections::HashMap;
use std::{future::Future, pin::Pin};

#[derive(Debug)]
pub struct FederationSourceRuntime {
  pub client: TracedHttpClient,
  pub identifier: String,
  pub config: FederationSourceConfig,
  pub supergraph: Supergraph,
}

#[cfg(target_arch = "wasm32")]
pub fn fetch_supergraph_schema(
  _url: &str,
  _headers: Option<&HashMap<String, String>>,
) -> Result<String, String> {
  panic!(
    "Remote supergraph source not supported in wasm32 at the moment, please fetch it from ENV"
  );
}

#[cfg(not(target_arch = "wasm32"))]
pub fn fetch_supergraph_schema(
  url: &str,
  headers: Option<&HashMap<String, String>>,
) -> Result<String, String> {
  let agent = ureq::Agent::new();

  let mut request = agent.request("POST", url);

  if let Some(headers_map) = headers {
    for (header_name, header_value) in headers_map {
      request = request.set(header_name, header_value);
    }
  }

  match request.call() {
    Ok(response) => {
      if response.status() == 200 {
        match response.into_string() {
          Ok(text) => Ok(text),
          Err(e) => Err(format!("Failed to read response text: {}", e)),
        }
      } else {
        Err(format!(
          "HTTP request failed with status: {}",
          response.status()
        ))
      }
    }
    Err(e) => Err(format!("HTTP request failed: {}", e)),
  }
}

#[tracing::instrument(level = "trace")]
pub fn load_supergraph(
  config: &FederationSourceConfig, // Add the config parameter here
) -> Result<Supergraph, Box<dyn std::error::Error>> {
  match &config.supergraph {
    SupergraphSourceConfig::File(file_ref) => {
      let content = std::fs::read_to_string(&file_ref.path)?;
      Ok(parse_supergraph(&content).unwrap())
    }
    SupergraphSourceConfig::EnvVar(env_var) => {
      let value = std::env::var(env_var)?;
      let decoded = engine::general_purpose::STANDARD_NO_PAD.decode(value)?;
      let content = String::from_utf8(decoded)?;
      Ok(parse_supergraph(&content).unwrap())
    }
    #[cfg(target_arch = "wasm32")]
    SupergraphSourceConfig::Remote {
      url: _,
      headers: _,
      fetch_every: _,
    } => {
      panic!(
        "Remote supergraph source not supported in wasm32 at the moment, please fetch it from ENV"
      );
    }
    #[cfg(not(target_arch = "wasm32"))]
    SupergraphSourceConfig::Remote {
      url,
      headers,
      fetch_every,
    } => {
      // Perform the initial fetch
      let supergraph_schema = fetch_supergraph_schema(url, headers.as_ref())?;
      let supergraph = parse_supergraph(&supergraph_schema).unwrap();

      // If `fetch_every` is set, start the periodic fetch
      if let Some(interval_str) = fetch_every {
        tracing::info!(
          "Registered supergraph schema fetch interval to update every: {interval_str}"
        );
        let client = wasm_polyfills::create_http_client()
          .build()
          .unwrap_or_else(|_| {
            // @expected: without a fetcher, there's no executor, without an executor, there's no gateway.
            panic!("Failed while initializing the executor's fetcher for Federation source");
          });

        let interval = humantime::parse_duration(interval_str)?;
        let mut runtime = FederationSourceRuntime {
          client: traced_reqwest(client),
          identifier: "test".to_string(),
          config: config.clone(),
          supergraph: supergraph.clone(),
        };
        let url = url.clone();
        let headers = headers.clone();
        tokio::spawn(async move {
          runtime.start_periodic_fetch(url, headers, interval).await;
        });
      }

      Ok(supergraph)
    }
  }
}

impl FederationSourceRuntime {
  pub fn new(identifier: String, config: FederationSourceConfig) -> Self {
    let client = wasm_polyfills::create_http_client()
      .build()
      .unwrap_or_else(|_| {
        // @expected: without a fetcher, there's no executor, without an executor, there's no gateway.
        panic!("Failed while initializing the executor's fetcher for Federation source");
      });

    let fetcher = traced_reqwest(client);

    let supergraph = match load_supergraph(&config) {
      Ok(e) => e,
      Err(e) => panic!("{e}"),
    };

    Self {
      client: fetcher,
      identifier,
      config,
      supergraph,
    }
  }

  pub async fn update_supergraph(&mut self, new_schema: String) {
    let new_supergraph = parse_supergraph(&new_schema).unwrap();
    self.supergraph = new_supergraph;
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub async fn start_periodic_fetch(
    &mut self,
    url: String,
    headers: Option<HashMap<String, String>>,
    interval: std::time::Duration,
  ) {
    let mut interval_timer = tokio::time::interval(interval);

    loop {
      interval_timer.tick().await;
      tracing::info!("Fetching new supergraph schema from {url}...");
      match fetch_supergraph_schema(&url, headers.as_ref()) {
        Ok(new_schema) => {
          self.update_supergraph(new_schema).await;
          tracing::info!("Successfully updated supergraph schema after being fetched from {url}");
        }
        Err(e) => eprintln!("Failed to fetch supergraph schema: {:?}", e),
      }
    }
  }
}

impl SourceRuntime for FederationSourceRuntime {
  fn name(&self) -> &str {
    &self.identifier
  }

  fn execute<'a>(
    &'a self,
    _route_data: &'a ConductorGatewayRouteData,
    request_context: &'a mut RequestExecutionContext,
  ) -> Pin<Box<(dyn Future<Output = Result<GraphQLResponse, SourceError>> + 'a)>> {
    Box::pin(wasm_polyfills::call_async(async move {
      let downstream_request = request_context
        .downstream_graphql_request
        .take()
        .expect("GraphQL request isn't available at the time of execution");

      //   let source_req = &mut downstream_request.request;

      // TODO: this needs to be called by conductor execution when fetching subgarphs
      //   route_data
      //     .plugin_manager
      //     .on_upstream_graphql_request(source_req)
      //     .await;

      let operation = downstream_request.parsed_operation;

      match execute_federation(&self.client, &self.supergraph, operation).await {
        Ok(response_data) => {
          let response = serde_json::from_str::<GraphQLResponse>(&response_data).unwrap();

          Ok(response)
        }
        Err(e) => Err(SourceError::UpstreamPlanningError(e)),
      }
    }))
  }
}
