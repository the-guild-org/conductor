use conductor_common::http::{ConductorHttpRequest, ConductorHttpResponse};
use conductor_common::plugin::{CreatablePlugin, Plugin, PluginError};
use conductor_common::vrl_functions::vrl_fns;
use tracing::{error, warn};
use vrl::compiler::{Function, Program, TypeState};

use conductor_common::execute::RequestExecutionContext;

use crate::config::VrlPluginConfig;

use super::downstream_graphql_request::vrl_downstream_graphql_request;
use super::downstream_http_request::vrl_downstream_http_request;
use super::downstream_http_response::vrl_downstream_http_response;
use super::upstream_http_request::vrl_upstream_http_request;

#[derive(Debug)]
pub struct VrlPlugin {
  pub(crate) on_downstream_http_request: Option<Program>,
  pub(crate) on_downstream_graphql_request: Option<Program>,
  pub(crate) on_downstream_http_response: Option<Program>,
  pub(crate) on_upstream_http_request: Option<Program>,
}

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for VrlPlugin {
  type Config = VrlPluginConfig;

  async fn create(config: Self::Config) -> Result<Box<Self>, PluginError> {
    let fns: Vec<Box<dyn Function>> = vrl_fns();

    let on_downstream_http_request = config
      .on_downstream_http_request
      .and_then(|cfg| VrlPlugin::compile_to_program(&fns, cfg.contents(), &Default::default()));
    let shared_state = on_downstream_http_request
      .as_ref()
      .map(|v| v.final_type_info().state)
      .unwrap_or_default();

    let on_downstream_graphql_request = config
      .on_downstream_graphql_request
      .and_then(|cfg| VrlPlugin::compile_to_program(&fns, cfg.contents(), &shared_state));
    let shared_state = VrlPlugin::merge_states(vec![
      shared_state,
      on_downstream_graphql_request
        .as_ref()
        .map(|v| v.final_type_info().state)
        .unwrap_or_default(),
    ]);
    let on_upstream_http_request = config
      .on_upstream_http_request
      .and_then(|cfg| VrlPlugin::compile_to_program(&fns, cfg.contents(), &shared_state));
    let shared_state = VrlPlugin::merge_states(vec![
      shared_state,
      on_upstream_http_request
        .as_ref()
        .map(|v| v.final_type_info().state)
        .unwrap_or_default(),
    ]);
    let on_downstream_http_response = config
      .on_downstream_http_response
      .and_then(|cfg| VrlPlugin::compile_to_program(&fns, cfg.contents(), &shared_state));

    Ok(Box::new(Self {
      on_downstream_http_request,
      on_downstream_graphql_request,
      on_upstream_http_request,
      on_downstream_http_response,
    }))
  }
}

// TODO: At some point, we can consider using the `self.program.info().target_queries/target_assignments`:
// it contains a list of properties used in the VRL programs, so we can efficiently create the context based only
// on the properties that are actually used. This is not a priority right now, but it's something to keep in mind if
// we want to improve performance.
#[async_trait::async_trait(?Send)]
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
    // TODO: handle errors nicely
    match vrl::compiler::compile_with_state(source, fns, parent_type_state, Default::default()) {
      Err(err) => {
        error!("vrl compiler error: {:?}", err);
        // @expected: if the provided VRL code in the config file can't compile, we have to exit.
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

  fn merge_states(states: Vec<TypeState>) -> TypeState {
    states
      .into_iter()
      .reduce(|a, b| a.merge(b))
      // @expected: `states` is a non-user provided variable
      .expect("can't merge states when `states` is an empty vector!")
  }
}
