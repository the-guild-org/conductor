use crate::schema_awareness::SchemaAwareness;
use conductor_common::execute::RequestExecutionContext;
use conductor_common::graphql::GraphQLResponse;
use conductor_common::plugin_manager::PluginManager;
use conductor_common::source::{GraphQLSourceInitError, SourceError, SourceRuntime};
use conductor_config::{FederationSourceConfig, SchemaAwarenessConfig};
use federation_query_planner::supergraph::{parse_supergraph, Supergraph};
use federation_query_planner::FederationExecutor;
use minitrace_reqwest::{traced_reqwest, TracedHttpClient};
use no_deadlocks::RwLock;
use std::sync::Arc;
use std::{future::Future, pin::Pin};

#[derive(Debug)]
pub struct FederationSourceRuntime {
  pub client: TracedHttpClient,
  pub identifier: String,
  pub config: FederationSourceConfig,
  pub schema_awareness: SchemaAwareness<Supergraph>,
}

impl FederationSourceRuntime {
  pub async fn new(
    identifier: String,
    config: FederationSourceConfig,
  ) -> Result<Self, GraphQLSourceInitError> {
    tracing::info!(
      "Initializing source '{}' of type 'graphql' with config: {:?}",
      identifier,
      config
    );

    let client = traced_reqwest(
      wasm_polyfills::create_http_client()
        .build()
        .map_err(|source| GraphQLSourceInitError::FetcherError { source })?,
    );

    let schema_awareness = SchemaAwareness::<Supergraph>::new(
      identifier.clone(),
      SchemaAwarenessConfig {
        format: conductor_config::SchemaAwarenessFormat::Sdl,
        on_error: conductor_config::SchemaAwarenessConfigOnError::Terminate,
        polling_interval: config.supergraph.polling_interval,
        source: config.supergraph.source.clone(),
      },
      |_, parsed| parse_supergraph(parsed),
    )
    .await
    .map_err(|source| GraphQLSourceInitError::SourceInitFailed {
      source: source.into(),
    })?;

    Ok(Self {
      schema_awareness,
      client,
      identifier,
      config,
    })
  }
}

impl SourceRuntime for FederationSourceRuntime {
  fn name(&self) -> &str {
    &self.identifier
  }

  fn schema(&self) -> Option<Arc<conductor_common::graphql::ParsedGraphQLSchema>> {
    self.schema_awareness.schema()
  }

  fn sdl(&self) -> Option<Arc<String>> {
    self.schema_awareness.raw()
  }

  fn execute<'a>(
    &'a self,
    plugin_manager: Arc<Box<dyn PluginManager>>,
    request_context: Arc<RwLock<RequestExecutionContext>>,
  ) -> Pin<Box<dyn Future<Output = Result<GraphQLResponse, SourceError>> + 'a>> {
    Box::pin(wasm_polyfills::call_async(async move {
      let operation = request_context
        .write()
        .unwrap()
        .downstream_graphql_request
        .take()
        .expect("GraphQL request isn't available at the time of execution")
        .parsed_operation;

      match self.schema_awareness.processed().as_ref() {
        Some(supergraph) => {
          let executor = FederationExecutor {
            client: &self.client,
            plugin_manager: plugin_manager.clone(),
            supergraph,
          };

          match executor
            .execute_federation(request_context, operation)
            .await
          {
            Ok((response_data, query_plan)) => {
              let mut response = serde_json::from_str::<GraphQLResponse>(&response_data).unwrap();

              if self.config.expose_query_plan {
                let mut ext = serde_json::Map::new();
                ext.insert(
                  "queryPlan".to_string(),
                  serde_json::value::to_value(query_plan).unwrap(),
                );

                response.append_extensions(ext);
              }

              Ok(response)
            }
            Err(e) => Err(SourceError::UpstreamPlanningError(e)),
          }
        }
        None => Err(SourceError::UpstreamPlanningError(anyhow::anyhow!(
          "Upstream planning error: schema awareness is not available!"
        ))),
      }
    }))
  }
}
