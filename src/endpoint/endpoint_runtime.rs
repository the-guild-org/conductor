use crate::{
    config::EndpointDefinition,
    plugins::{flow_context::FlowContext, plugin_manager::PluginManager},
    source::base_source::{SourceError, SourceRequest, SourceService},
};
use axum::body::Body;
use serde_json::json;
use std::sync::Arc;

pub type EndpointResponse = hyper::Response<Body>;

#[derive(Debug)]
pub enum EndpointError {
    UpstreamError(SourceError),
    SourceQueryNotAvailable,
}

impl From<EndpointError> for hyper::Response<Body> {
    fn from(value: EndpointError) -> Self {
        match value {
            EndpointError::UpstreamError(_e) => hyper::Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(
                    json!({"error": "upstream is not healthy"}).to_string(),
                ))
                .unwrap(),
            EndpointError::SourceQueryNotAvailable => hyper::Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(
                    json!({"error": "source query was not extracted from incoming request"})
                        .to_string(),
                ))
                .unwrap(),
        }
    }
}

#[derive(Clone)]
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

    pub async fn handle_request(
        &self,
        flow_ctx: FlowContext,
    ) -> (FlowContext, Result<hyper::Response<Body>, EndpointError>) {
        match flow_ctx.downstream_graphql_request.as_ref() {
            Some(source_request) => {
                // DOTAN: Can we avoid cloning here?
                let upstream_request = SourceRequest::from_parts(
                    source_request.operation_name.as_deref(),
                    source_request.query.as_ref(),
                    Some(&source_request.variables),
                );

                let source_result = self.upstream.call(upstream_request);

                // DOTAN: We probably need some kind of handling for network-related errors here,
                // I guess some kind of static "upstream is not healthy" error response?
                match source_result.await {
                    Ok(source_response) => (flow_ctx, Ok(source_response)),
                    Err(e) => (flow_ctx, Err(EndpointError::UpstreamError(e))),
                }
            }
            None => (flow_ctx, Err(EndpointError::SourceQueryNotAvailable)),
        }
    }
}
