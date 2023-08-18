use std::fmt::Debug;

use crate::source::base_source::SourceRequest;

use super::flow_context::FlowContext;

pub trait Plugin: Sync + Send {
    fn on_downstream_http_request(&self, mut _ctx: FlowContext) -> FlowContext {
        _ctx
    }
    fn on_downstream_http_response(&self, mut _ctx: FlowContext) -> FlowContext {
        _ctx
    }
    fn on_downstream_graphql_request(&self, mut _ctx: FlowContext) -> FlowContext {
        _ctx
    }
    fn on_upstream_graphql_request(&self, mut _req: SourceRequest) -> SourceRequest {
        _req
    }
}

impl Debug for dyn Plugin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Plugin")
    }
}
