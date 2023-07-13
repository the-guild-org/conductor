use tracing::debug;

use super::{core::Plugin, flow_context::FlowContext};

pub struct VerboseLoggingPlugin {}

impl Plugin for VerboseLoggingPlugin {
    fn on_downstream_graphql_request(&self, mut _ctx: FlowContext) -> FlowContext {
        debug!("on_downstream_graphql_request, ctx: {:?}", _ctx);
        _ctx
    }

    fn on_downstream_http_response(&self, mut _ctx: FlowContext) -> FlowContext {
        debug!("on_downstream_http_response, ctx: {:?}", _ctx);
        _ctx
    }

    fn on_downstream_http_request(&self, mut _ctx: FlowContext) -> FlowContext {
        debug!("on_downstream_http_request, ctx: {:?}", _ctx);
        _ctx
    }

    fn on_upstream_graphql_request(
        &self,
        mut _req: crate::source::base_source::SourceRequest,
    ) -> crate::source::base_source::SourceRequest {
        debug!("on_upstream_graphql_request, req: {:?}", _req);
        _req
    }
}
