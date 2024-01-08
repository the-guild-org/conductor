use std::{fmt::Debug, future::Future, pin::Pin};

use crate::gateway::ConductorGatewayRouteData;
use conductor_common::{
  execute::RequestExecutionContext, graphql::GraphQLResponse, http::StatusCode,
};

pub trait SourceRuntime: Debug + Send + Sync + 'static {
  fn execute<'a>(
    &'a self,
    _route_data: &'a ConductorGatewayRouteData,
    _request_context: &'a mut RequestExecutionContext,
  ) -> Pin<Box<(dyn Future<Output = Result<GraphQLResponse, SourceError>> + 'a)>>;

  fn name(&self) -> &str;
}

#[derive(thiserror::Error, Debug)]
pub enum SourceError {
  #[error("unexpected HTTP status: {0}")]
  UnexpectedHTTPStatusError(StatusCode),
  #[error("short circuit")]
  ShortCircuit,
  #[error("network error: {0}")]
  NetworkError(reqwest_middleware::Error),
  #[error("upstream planning error: {0}")]
  UpstreamPlanningError(anyhow::Error),
}

impl SourceError {
  pub fn http_status_code(&self) -> StatusCode {
    match self {
      Self::UnexpectedHTTPStatusError(_) => StatusCode::BAD_GATEWAY,
      Self::ShortCircuit => StatusCode::INTERNAL_SERVER_ERROR,
      Self::NetworkError(_) => StatusCode::BAD_GATEWAY,
      Self::UpstreamPlanningError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
  }
}

impl From<SourceError> for GraphQLResponse {
  fn from(error: SourceError) -> Self {
    GraphQLResponse::new_error_with_code(&error.to_string(), error.http_status_code())
  }
}
