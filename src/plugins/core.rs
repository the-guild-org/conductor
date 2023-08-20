use std::fmt::Debug;

use axum::Router;
use hyper::Body;

use crate::{endpoint::endpoint_runtime::EndpointError, source::base_source::SourceRequest};

use super::flow_context::FlowContext;

pub trait Plugin: Sync + Send {
    fn on_endpoint_creation(&self, _router: Router<()>) -> axum::Router<()> {
        _router
    }
    fn on_downstream_http_request(&self, _ctx: &mut FlowContext) {}
    fn on_downstream_http_response(&self, _ctx: &mut FlowContext) {}
    fn on_downstream_graphql_request(&self, _ctx: &mut FlowContext) {}
    fn on_upstream_graphql_request(&self, _req: &mut SourceRequest) {}
    fn on_upstream_graphql_response(
        &self,
        _response: &mut Result<hyper::Response<Body>, EndpointError>,
    ) {
    }
}

impl Debug for dyn Plugin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Plugin")
    }
}
