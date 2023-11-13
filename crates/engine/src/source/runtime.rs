use conductor_common::{graphql::GraphQLResponse, http::StatusCode};

use crate::{
    gateway::ConductorGatewayRouteData, request_execution_context::RequestExecutionContext,
};

#[async_trait::async_trait]
pub trait SourceRuntime: Send + Sync + 'static {
    async fn execute(
        &self,
        _route_data: &ConductorGatewayRouteData,
        _request_context: &mut RequestExecutionContext<'_>,
    ) -> Result<GraphQLResponse, SourceError>;
}

#[derive(thiserror::Error, Debug)]
pub enum SourceError {
    #[error("unexpected HTTP status: {0}")]
    UnexpectedHTTPStatusError(StatusCode),
    #[error("network error: {0}")]
    NetworkError(reqwest::Error),
}

impl From<SourceError> for GraphQLResponse {
    fn from(error: SourceError) -> Self {
        GraphQLResponse::new_error(&error.to_string())
    }
}
