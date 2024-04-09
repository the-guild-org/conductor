use std::sync::Arc;

use crate::{
  execute::RequestExecutionContext,
  graphql::GraphQLRequest,
  http::{ConductorHttpRequest, ConductorHttpResponse},
  source::SourceRuntime,
};
use no_deadlocks::RwLock;
use reqwest::Response;

#[async_trait::async_trait(?Send)]
pub trait PluginManager: std::fmt::Debug + Send + Sync {
  async fn on_downstream_http_request(&self, context: Arc<RwLock<RequestExecutionContext>>);
  async fn on_downstream_http_response(
    &self,
    context: Arc<RwLock<RequestExecutionContext>>,
    response: &mut ConductorHttpResponse,
  );
  async fn on_downstream_graphql_request(
    &self,
    source_runtime: Arc<Box<dyn SourceRuntime>>,
    context: Arc<RwLock<RequestExecutionContext>>,
  );
  async fn on_upstream_graphql_request(&self, req: &mut GraphQLRequest);
  async fn on_upstream_http_request(
    &self,
    context: Arc<RwLock<RequestExecutionContext>>,
    request: &mut ConductorHttpRequest,
  );
  async fn on_upstream_http_response(
    &self,
    context: Arc<RwLock<RequestExecutionContext>>,
    response: &Result<Response, reqwest_middleware::Error>,
  );
}
