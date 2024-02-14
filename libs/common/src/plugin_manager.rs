use crate::{
  execute::RequestExecutionContext,
  graphql::GraphQLRequest,
  http::{ConductorHttpRequest, ConductorHttpResponse},
};
use reqwest::Response;

#[async_trait::async_trait(?Send)]
pub trait PluginManager: std::fmt::Debug + Send + Sync {
  async fn on_downstream_http_request(&self, context: &mut RequestExecutionContext);
  fn on_downstream_http_response(
    &self,
    context: &mut RequestExecutionContext,
    response: &mut ConductorHttpResponse,
  );
  async fn on_downstream_graphql_request(&self, context: &mut RequestExecutionContext);
  async fn on_upstream_graphql_request<'a>(&self, req: &mut GraphQLRequest);
  async fn on_upstream_http_request<'a>(
    &self,
    ctx: &mut RequestExecutionContext,
    request: &mut ConductorHttpRequest,
  );
  async fn on_upstream_http_response<'a>(
    &self,
    ctx: &mut RequestExecutionContext,
    response: &Result<Response, reqwest_middleware::Error>,
  );
}
