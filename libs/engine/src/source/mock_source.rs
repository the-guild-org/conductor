use std::sync::Arc;

use conductor_common::{
  execute::RequestExecutionContext, graphql::GraphQLResponse, plugin_manager::PluginManager,
};
use conductor_config::MockedSourceConfig;

use conductor_common::source::SourceRuntime;
use no_deadlocks::RwLock;

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

  fn schema(&self) -> Option<std::sync::Arc<conductor_common::graphql::ParsedGraphQLSchema>> {
    None
  }

  fn sdl(&self) -> Option<std::sync::Arc<String>> {
    None
  }

  fn execute<'a>(
    &'a self,
    _plugin_manager: Arc<Box<dyn PluginManager>>,
    _request_context: Arc<RwLock<RequestExecutionContext>>,
  ) -> std::pin::Pin<
    Box<
      (dyn futures::prelude::Future<
        Output = Result<
          conductor_common::graphql::GraphQLResponse,
          conductor_common::source::SourceError,
        >,
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
