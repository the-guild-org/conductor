use std::sync::Arc;

use conductor_common::{
  execute::RequestExecutionContext,
  graphql::{validate_graphql_operation, GraphQLResponse},
  plugin::{CreatablePlugin, Plugin, PluginError},
  source::SourceRuntime,
};

use crate::config::GraphQLValidationPluginConfig;

#[derive(Debug)]
pub struct GraphQLValidationPlugin(GraphQLValidationPluginConfig);

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for GraphQLValidationPlugin {
  type Config = GraphQLValidationPluginConfig;

  async fn create(config: Self::Config) -> Result<Box<Self>, PluginError> {
    Ok(Box::new(Self(config)))
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for GraphQLValidationPlugin {
  async fn on_downstream_graphql_request(
    &self,
    source_runtime: Arc<Box<dyn SourceRuntime>>,
    request_context: &mut RequestExecutionContext,
  ) {
    if let Some(operation) = &request_context.downstream_graphql_request {
      if let Some(schema) = source_runtime.schema() {
        let errors = validate_graphql_operation(schema.as_ref(), &operation.parsed_operation);

        if !errors.is_empty() {
          let gql_response: GraphQLResponse = errors.into();
          request_context.short_circuit(gql_response.into());
        }
      } else {
        tracing::warn!(
          "Plugin graphql_validation is enabled, but source does not have a scheme awareness available. Skipping."
        );
      }
    }
  }
}
