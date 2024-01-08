use conductor_common::graphql::GraphQLResponse;
use conductor_config::MockedSourceConfig;

use super::runtime::SourceRuntime;

#[derive(Debug)]
pub struct MockedSourceRuntime {
  pub config: MockedSourceConfig,
  pub identifier: String,
}

impl MockedSourceRuntime {
  pub fn new(identifier: String, config: MockedSourceConfig) -> Self {
    Self { config, identifier }
  }
}

impl SourceRuntime for MockedSourceRuntime {
  fn name(&self) -> &str {
    &self.identifier
  }

  fn execute<'a>(
    &'a self,
    _route_data: &'a crate::gateway::ConductorGatewayRouteData,
    _request_context: &'a mut conductor_common::execute::RequestExecutionContext,
  ) -> std::pin::Pin<
    Box<
      (dyn futures::prelude::Future<
        Output = Result<conductor_common::graphql::GraphQLResponse, super::runtime::SourceError>,
      > + 'a),
    >,
  > {
    Box::pin(wasm_polyfills::call_async(async move {
      Ok(
        serde_json::from_slice::<GraphQLResponse>(self.config.response_data.contents.as_bytes())
          .unwrap_or_else(|e| GraphQLResponse::new_error(&e.to_string())),
      )
    }))
  }
}
