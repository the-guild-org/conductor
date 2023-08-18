use tracing::debug;

use crate::source::base_source::SourceRequest;

use super::{core::Plugin, flow_context::FlowContext};

pub struct VerboseLoggingPlugin {}

impl Plugin for VerboseLoggingPlugin {
    fn on_downstream_graphql_request(&self, ctx: &mut FlowContext) {
        debug!("on_downstream_graphql_request, ctx: {:?}", ctx);
    }

    fn on_downstream_http_response(&self, ctx: &mut FlowContext) {
        debug!("on_downstream_http_response, ctx: {:?}", ctx);
    }

    fn on_downstream_http_request(&self, ctx: &mut FlowContext) {
        debug!("on_downstream_http_request, ctx: {:?}", ctx);
    }

    fn on_upstream_graphql_request<'a>(&self, req: &mut SourceRequest<'a>) {
        debug!("on_upstream_graphql_request, req: {:?}", req);
    }
}
