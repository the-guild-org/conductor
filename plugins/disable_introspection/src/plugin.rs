use crate::config::DisableIntrospectionPluginConfig;
use conductor_common::{
  graphql::GraphQLResponse,
  http::StatusCode,
  plugin::{CreatablePlugin, Plugin, PluginError},
};
use tracing::{error, warn};
use vrl::{
  compiler::{Context, Program, TargetValue, TimeZone},
  value,
  value::Secrets,
};

use conductor_common::execute::RequestExecutionContext;

use vrl_plugin::{utils::conductor_request_to_value, vrl_functions::vrl_fns};

#[derive(Debug)]
pub struct DisableIntrospectionPlugin {
  condition: Option<Program>,
}

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for DisableIntrospectionPlugin {
  type Config = DisableIntrospectionPluginConfig;

  async fn create(config: Self::Config) -> Result<Box<dyn Plugin>, PluginError> {
    let instance = match &config.condition {
      Some(condition) => match vrl::compiler::compile(condition.contents(), &vrl_fns()) {
        Err(err) => {
          error!("vrl compiler error: {:?}", err);
          panic!("failed to compile vrl program for disable_introspection plugin");
        }
        Ok(result) => {
          if result.warnings.len() > 0 {
            warn!(
              "vrl compiler warning for disable_introspection plugin: {:?}",
              result.warnings
            );
          }

          Self {
            condition: Some(result.program),
          }
        }
      },
      None => Self { condition: None },
    };

    Ok(Box::new(instance))
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for DisableIntrospectionPlugin {
  async fn on_downstream_graphql_request(&self, ctx: &mut RequestExecutionContext) {
    if let Some(op) = &ctx.downstream_graphql_request {
      if op.is_introspection_query() {
        let should_disable = match &self.condition {
          Some(program) => {
            let downstream_http_req = conductor_request_to_value(&ctx.downstream_http_request);
            let mut target = TargetValue {
              value: value!({}),
              metadata: value!({
                downstream_http_req: downstream_http_req,
              }),
              secrets: Secrets::default(),
            };

            match program.resolve(&mut Context::new(
              &mut target,
              ctx.vrl_shared_state(),
              &TimeZone::default(),
            )) {
              Ok(ret) => match ret {
                vrl::value::Value::Boolean(b) => b,
                _ => {
                  error!("DisableIntrospectionPlugin::vrl::condition must return a boolean, but returned a non-boolean value: {:?}, ignoring...", ret);

                  true
                }
              },
              Err(err) => {
                error!(
                  "DisableIntrospectionPlugin::vrl::condition resolve error: {:?}",
                  err
                );

                ctx.short_circuit(
                  GraphQLResponse::new_error("vrl runtime error")
                    .into_with_status_code(StatusCode::BAD_GATEWAY),
                );
                return;
              }
            }
          }
          None => true,
        };

        if should_disable {
          ctx.short_circuit(GraphQLResponse::new_error("Introspection is disabled").into());
        }
      }
    }
  }
}
