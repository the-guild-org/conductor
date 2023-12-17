use conductor_common::http::{ConductorHttpRequest, ConductorHttpResponse};
use conductor_config::plugins::{VrlConfigReference, VrlPluginConfig};
use tracing::{error, warn};
use vrl::compiler::{Function, Program, TypeState};

use crate::plugins::core::Plugin;
use crate::request_execution_context::RequestExecutionContext;

use super::downstream_graphql_request::vrl_downstream_graphql_request;
use super::downstream_http_request::vrl_downstream_http_request;
use super::downstream_http_response::vrl_downstream_http_response;
use super::upstream_http_request::vrl_upstream_http_request;
use super::vrl_functions::vrl_fns;

pub struct VrlPlugin {
    pub(crate) on_downstream_http_request: Option<Program>,
    pub(crate) on_downstream_graphql_request: Option<Program>,
    pub(crate) on_downstream_http_response: Option<Program>,
    pub(crate) on_upstream_http_request: Option<Program>,
}

// TODO: At some point, we can consider using the `self.program.info().target_queries/target_assignments`:
// it contains a list of properties used in the VRL programs, so we can efficiently create the context based only
// on the properties that are actually used. This is not a priority right now, but it's something to keep in mind if
// we want to improve performance.
#[async_trait::async_trait]
impl Plugin for VrlPlugin {
    async fn on_downstream_http_request(&self, ctx: &mut RequestExecutionContext) {
        if let Some(program) = &self.on_downstream_http_request {
            vrl_downstream_http_request(program, ctx);
        }
    }

    async fn on_downstream_graphql_request(&self, ctx: &mut RequestExecutionContext) {
        if let Some(program) = &self.on_downstream_graphql_request {
            vrl_downstream_graphql_request(program, ctx);
        }
    }

    async fn on_upstream_http_request(
        &self,
        ctx: &mut RequestExecutionContext,
        req: &mut ConductorHttpRequest,
    ) {
        if let Some(program) = &self.on_upstream_http_request {
            vrl_upstream_http_request(program, ctx, req);
        }
    }

    fn on_downstream_http_response(
        &self,
        ctx: &mut RequestExecutionContext,
        response: &mut ConductorHttpResponse,
    ) {
        if let Some(program) = &self.on_downstream_http_response {
            vrl_downstream_http_response(program, ctx, response);
        }
    }
}

impl VrlPlugin {
    fn compile_to_program(
        fns: &[Box<dyn Function>],
        source: &str,
        parent_type_state: &TypeState,
    ) -> Option<Program> {
        match vrl::compiler::compile_with_state(source, fns, parent_type_state, Default::default())
        {
            Err(err) => {
                error!("vrl compiler error: {:?}", err);
                panic!("failed to compile vrl program");
            }
            Ok(result) => {
                if result.warnings.len() > 0 {
                    warn!("vrl compiler warning: {:?}", result.warnings);
                }

                Some(result.program)
            }
        }
    }

    fn config_to_contents(config: &VrlConfigReference) -> &String {
        match config {
            VrlConfigReference::Inline { content } => content,
            VrlConfigReference::File { path } => &path.contents,
        }
    }

    fn merge_states(states: Vec<TypeState>) -> TypeState {
        states.into_iter().reduce(|a, b| a.merge(b)).unwrap()
    }

    pub fn new(config: VrlPluginConfig) -> Self {
        let fns: Vec<Box<dyn Function>> = vrl_fns();

        let on_downstream_http_request = config.on_downstream_http_request.and_then(|cfg| {
            VrlPlugin::compile_to_program(
                &fns,
                VrlPlugin::config_to_contents(&cfg),
                &Default::default(),
            )
        });
        let shared_state = on_downstream_http_request
            .as_ref()
            .map(|v| v.final_type_info().state)
            .unwrap_or_default();

        let on_downstream_graphql_request = config.on_downstream_graphql_request.and_then(|cfg| {
            VrlPlugin::compile_to_program(&fns, VrlPlugin::config_to_contents(&cfg), &shared_state)
        });
        let shared_state = VrlPlugin::merge_states(vec![
            shared_state,
            on_downstream_graphql_request
                .as_ref()
                .map(|v| v.final_type_info().state)
                .unwrap_or_default(),
        ]);
        let on_upstream_http_request = config.on_upstream_http_request.and_then(|cfg| {
            VrlPlugin::compile_to_program(&fns, VrlPlugin::config_to_contents(&cfg), &shared_state)
        });
        let shared_state = VrlPlugin::merge_states(vec![
            shared_state,
            on_upstream_http_request
                .as_ref()
                .map(|v| v.final_type_info().state)
                .unwrap_or_default(),
        ]);
        let on_downstream_http_response = config.on_downstream_http_response.and_then(|cfg| {
            VrlPlugin::compile_to_program(&fns, VrlPlugin::config_to_contents(&cfg), &shared_state)
        });

        Self {
            on_downstream_http_request,
            on_downstream_graphql_request,
            on_upstream_http_request,
            on_downstream_http_response,
        }
    }
}
