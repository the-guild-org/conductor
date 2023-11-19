use std::fmt::Debug;

use conductor_common::{
    graphql::{GraphQLRequest, GraphQLResponse},
    http::ConductorHttpResponse,
};

use crate::request_execution_context::RequestExecutionContext;

#[async_trait::async_trait]
pub trait Plugin: Sync + Send {
    // An HTTP request send from the client to Conductor
    async fn on_downstream_http_request(&self, _ctx: &mut RequestExecutionContext) {}
    // A final HTTP response send from Conductor to the client
    fn on_downstream_http_response(
        &self,
        _ctx: &RequestExecutionContext,
        _response: &mut ConductorHttpResponse,
    ) {
    }
    // An incoming GraphQL operation executed to Conductor
    async fn on_downstream_graphql_request(&self, _ctx: &mut RequestExecutionContext) {}
    // A request send from Conductor to the upstream GraphQL server
    async fn on_upstream_graphql_request(&self, _req: &mut GraphQLRequest) {}
    // A response sent from the upstream GraphQL server to Conductor
    async fn on_upstream_graphql_response(&self, _response: &mut GraphQLResponse) {}
}

impl Debug for dyn Plugin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Plugin")
    }
}
