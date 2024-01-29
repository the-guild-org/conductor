use std::sync::Arc;

use crate::config::DisableIntrospectionPluginConfig;
use conductor_common::{
  graphql::GraphQLResponse,
  http::StatusCode,
  logging_locks::LoggingRwLock,
  plugin::{CreatablePlugin, Plugin, PluginError},
  source::SourceRuntime,
  vrl_utils::{conductor_request_to_value, VrlProgramProxy},
};
use tracing::error;
use vrl::value;

use conductor_common::execute::RequestExecutionContext;

#[derive(Debug)]
pub struct DisableIntrospectionPlugin {
  condition: Option<VrlProgramProxy>,
}

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for DisableIntrospectionPlugin {
  type Config = DisableIntrospectionPluginConfig;

  async fn create(config: Self::Config) -> Result<Box<Self>, PluginError> {
    let condition = match &config.condition {
      Some(condition) => match condition.program() {
        Ok(program) => Some(program),
        Err(e) => {
          return Err(PluginError::InitError {
            source: anyhow::anyhow!("vrl compiler error: {:?}", e),
          })
        }
      },
      None => None,
    };

    Ok(Box::new(Self { condition }))
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for DisableIntrospectionPlugin {
  async fn on_downstream_graphql_request(
    &self,
    _source_runtime: Arc<Box<dyn SourceRuntime>>,
    ctx: Arc<LoggingRwLock<RequestExecutionContext>>,
  ) {
    if let Some(op) = &ctx.read().await.downstream_graphql_request {
      if op.is_introspection_query() {
        let should_disable = match &self.condition {
          Some(program) => {
            let downstream_http_req =
              conductor_request_to_value(&ctx.read().await.downstream_http_request);

            match program.resolve_with_state(
              value::Value::Null,
              value!({
                downstream_http_req: downstream_http_req,
              }),
              ctx.write().await.vrl_shared_state(),
            ) {
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

                ctx.write().await.short_circuit(
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
          ctx
            .write()
            .await
            .short_circuit(GraphQLResponse::new_error("Introspection is disabled").into());
        }
      }
    }
  }
}
