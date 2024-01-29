use std::{fmt::Debug, future::Future, pin::Pin, sync::Arc};

use crate::{
  execute::RequestExecutionContext,
  graphql::{GraphQLResponse, ParsedGraphQLSchema},
  http::StatusCode,
  logging_locks::LoggingRwLock,
  plugin_manager::PluginManager,
};

#[derive(thiserror::Error, Debug)]
pub enum GraphQLSourceInitError {
  #[error("failed to init source")]
  SourceInitFailed { source: anyhow::Error },
  #[error("failed to init http client")]
  FetcherError { source: reqwest::Error },
}

pub trait SourceRuntime: Debug + Send + Sync + 'static {
  fn execute<'a>(
    &'a self,
    _plugin_manager: Arc<Box<dyn PluginManager>>,
    _request_context: Arc<LoggingRwLock<RequestExecutionContext>>,
  ) -> Pin<Box<(dyn Future<Output = Result<GraphQLResponse, SourceError>> + 'a)>>;

  fn name(&self) -> &str;
  fn schema(&self) -> Option<Arc<ParsedGraphQLSchema>>;
  fn sdl(&self) -> Option<Arc<String>>;
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
