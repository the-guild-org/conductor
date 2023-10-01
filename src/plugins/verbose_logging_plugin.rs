use axum::body::BoxBody;
use tracing::debug;

use crate::graphql_utils::GraphQLRequest;

use super::{core::Plugin, flow_context::FlowContext};

pub struct VerboseLoggingPlugin {}

impl Plugin for VerboseLoggingPlugin {
    fn on_downstream_graphql_request(&self, ctx: &mut FlowContext) {
        debug!("on_downstream_graphql_request, ctx: {:?}", ctx);
    }

    fn on_downstream_http_response(
        &self,
        ctx: &FlowContext,
        response: &mut http::Response<BoxBody>,
    ) {
        debug!(
            "on_downstream_http_response, ctx: {:?}, response: {:?}",
            ctx, response
        );
    }

    fn on_downstream_http_request(&self, ctx: &mut FlowContext) {
        debug!("on_downstream_http_request, ctx: {:?}", ctx);
    }

    fn on_upstream_graphql_request(&self, req: &mut GraphQLRequest) {
        debug!("on_upstream_graphql_request, req: {:?}", req);
    }
}
