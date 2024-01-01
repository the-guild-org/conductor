use std::{fmt::Debug, future::Future, pin::Pin};

use conductor_common::{
  execute::RequestExecutionContext, graphql::GraphQLResponse, http::StatusCode,
};

use crate::gateway::ConductorGatewayRouteData;

pub trait SourceRuntime: Debug + Send + Sync + 'static {
  fn execute<'a>(
    &'a self,
    _route_data: &'a ConductorGatewayRouteData,
    _request_context: &'a mut RequestExecutionContext,
  ) -> Pin<Box<(dyn Future<Output = Result<GraphQLResponse, SourceError>> + 'a)>>;
}

#[derive(thiserror::Error, Debug)]
pub enum SourceError {
  #[error("unexpected HTTP status: {0}")]
  UnexpectedHTTPStatusError(StatusCode),
  #[error("short circuit")]
  ShortCircuit,
  #[error("network error: {0}")]
  NetworkError(reqwest::Error),
}

impl From<SourceError> for GraphQLResponse {
  fn from(error: SourceError) -> Self {
    GraphQLResponse::new_error(&error.to_string())
  }
}
