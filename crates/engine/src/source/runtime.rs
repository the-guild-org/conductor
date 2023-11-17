use std::{future::Future, pin::Pin};

use conductor_common::{graphql::GraphQLResponse, http::StatusCode};

use crate::{
    gateway::ConductorGatewayRouteData, request_execution_context::RequestExecutionContext,
};

pub trait SourceRuntime: Send + Sync + 'static {
    fn execute<'a>(
        &'a self,
        _route_data: &'a ConductorGatewayRouteData,
        _request_context: &'a mut RequestExecutionContext<'_>,
    ) -> Pin<Box<(dyn Future<Output = Result<GraphQLResponse, SourceError>> + Send + 'a)>>;
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
