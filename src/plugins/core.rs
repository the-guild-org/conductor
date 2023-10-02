use std::fmt::Debug;

use axum::{body::BoxBody, Router};
use hyper::Body;

use crate::{endpoint::endpoint_runtime::EndpointError, graphql_utils::GraphQLRequest};

use super::flow_context::FlowContext;

#[async_trait::async_trait]
pub trait Plugin: Sync + Send {
    fn on_endpoint_creation(&self, _router: Router<()>) -> axum::Router<()> {
        _router
    }
    // An HTTP request send from the client to Conductor
    async fn on_downstream_http_request(&self, _ctx: &mut FlowContext) {}
    // A final HTTP response send from Conductor to the client
    fn on_downstream_http_response(
        &self,
        _ctx: &FlowContext,
        _response: &mut http::Response<BoxBody>,
    ) {
    }
    // An incoming GraphQL operation executed to Conductor
    async fn on_downstream_graphql_request(&self, _ctx: &mut FlowContext) {}
    // A request send from Conductor to the upstream GraphQL server
    async fn on_upstream_graphql_request(&self, _req: &mut GraphQLRequest) {}
    // A response sent from the upstream GraphQL server to Conductor
    async fn on_upstream_graphql_response(
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
