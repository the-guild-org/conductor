use std::{fmt::Debug, sync::Arc};

use crate::{
  graphql::GraphQLRequest,
  http::{ConductorHttpRequest, ConductorHttpResponse},
  source::SourceRuntime,
};
use no_deadlocks::RwLock;
use reqwest::Response;

use crate::execute::RequestExecutionContext;

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
  #[error("Plugin init error: {source}")]
  InitError { source: anyhow::Error },
  #[error("Plugin \"{name}\" is not supported in the current runtime.")]
  PluginNotSupportedInRuntime { name: String },
}

#[async_trait::async_trait(?Send)]
pub trait CreatablePlugin: Plugin {
  type Config;

  async fn create(config: Self::Config) -> Result<Box<Self>, PluginError>;
}

#[async_trait::async_trait(?Send)]
pub trait Plugin: Sync + Send + Debug {
  // From: on_downstream_http_request -> on_downstream_graphql_request -> on_upstream_graphql_request -> on_upstream_http_request
  // To: on_upstream_http_response -> on_downstream_graphql_response -> on_downstream_http_response
  // Step 1: An HTTP request send from the client to Conductor
  async fn on_downstream_http_request(&self, _ctx: Arc<RwLock<RequestExecutionContext>>) {}
  // Step 2: An incoming GraphQL operation executed to Conductor
  async fn on_downstream_graphql_request(
    &self,
    _source_runtime: Arc<Box<dyn SourceRuntime>>,
    _ctx: Arc<RwLock<RequestExecutionContext>>,
  ) {
  }
  // Step 3: A GraphQL request send from Conductor to the upstream GraphQL server
  async fn on_upstream_graphql_request(&self, _req: &mut GraphQLRequest) {}
  // Step 4: A GraphQL request send from Conductor to the upstream GraphQL server
  async fn on_upstream_http_request(
    &self,
    _ctx: Arc<RwLock<RequestExecutionContext>>,
    _req: &mut ConductorHttpRequest,
  ) {
  }
  // Step 5: We got a response from the upstream server
  async fn on_upstream_http_response(
    &self,
    _ctx: Arc<RwLock<RequestExecutionContext>>,
    _res: &Result<Response, reqwest_middleware::Error>,
  ) {
  }
  // Step 6: A final HTTP response send from Conductor to the client
  async fn on_downstream_http_response(
    &self,
    _ctx: Arc<RwLock<RequestExecutionContext>>,
    _response: &mut ConductorHttpResponse,
  ) {
  }
}
