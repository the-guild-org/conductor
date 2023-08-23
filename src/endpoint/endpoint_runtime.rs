use crate::{
    config::EndpointDefinition,
    graphql_utils::GraphQLResponse,
    plugins::{flow_context::FlowContext, plugin_manager::PluginManager},
    source::base_source::{SourceError, SourceService},
};
use axum::{
    body::{Body, BoxBody},
    response::IntoResponse,
};
use http::StatusCode;
use std::sync::Arc;

use super::graphiql::GraphiQLSource;

pub type EndpointResponse = hyper::Response<Body>;

#[derive(Debug)]
pub enum EndpointError {
    UpstreamError(SourceError),
}

impl IntoResponse for EndpointError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, error_message) = match self {
            EndpointError::UpstreamError(e) => (
                StatusCode::BAD_GATEWAY,
                format!("Invalid GraphQL variables JSON format: {:?}", e),
            ),
        };

        let gql_response = GraphQLResponse::new_error(&error_message);
        gql_response.into_response(status_code)
    }
}

#[derive(Clone, Debug)]
pub struct EndpointRuntime {
    pub config: EndpointDefinition,
    pub plugin_manager: Arc<PluginManager>,
    pub upstream: Arc<Box<dyn SourceService + Send>>,
}

impl EndpointRuntime {
    pub fn new(
        endpoint_config: EndpointDefinition,
        source: impl SourceService,
        plugin_manager: Arc<PluginManager>,
    ) -> Self {
        Self {
            config: endpoint_config,
            upstream: Arc::new(Box::new(source)),
            plugin_manager,
        }
    }

    pub fn compose_graphiql(&self) -> GraphiQLSource {
        GraphiQLSource::new(&self.config.path)
    }

    pub async fn handle_request<'a>(
        &self,
        flow_ctx: FlowContext<'a>,
    ) -> (
        FlowContext<'a>,
        Result<hyper::Response<BoxBody>, EndpointError>,
    ) {
        let graphql_req = flow_ctx
            .downstream_graphql_request
            .as_ref()
            .unwrap()
            .request
            .clone();
        let source_result = self.upstream.call(graphql_req);

        match source_result.await {
            Ok(source_response) => (flow_ctx, Ok(source_response.into_response())),
            Err(e) => (flow_ctx, Err(EndpointError::UpstreamError(e))),
        }
    }
}
