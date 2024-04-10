use std::str::FromStr;

use crate::config::{HeaderManipulationAction, HeaderManipulationPluginConfig};

use conductor_common::execute::RequestExecutionContext;
use conductor_common::http::ConductorHttpRequest;
use conductor_common::plugin::{CreatablePlugin, Plugin, PluginError};
use reqwest::header::{HeaderName, HeaderValue};

#[derive(Debug)]
pub struct HeaderManipulationPlugin(HeaderManipulationPluginConfig);

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for HeaderManipulationPlugin {
  type Config = HeaderManipulationPluginConfig;

  async fn create(config: Self::Config) -> Result<Box<Self>, PluginError> {
    Ok(Box::new(Self(config)))
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for HeaderManipulationPlugin {
  async fn on_upstream_http_request(
    &self,
    ctx: &mut RequestExecutionContext,
    req: &mut ConductorHttpRequest,
  ) {
    for action in &self.0.upstream {
      match action {
        HeaderManipulationAction::Copy { to, from } => {
          if let Some(header) = ctx.downstream_http_request.headers.get(from) {
            match HeaderName::from_str(to) {
              Ok(header_name) => {
                req.headers.insert(header_name, header.clone());
              }
              Err(_) => {
                tracing::warn!(
                  "Plugin header_manipulation failed to parse header name '{:?}'",
                  to
                );
              }
            }
          }
        }
        HeaderManipulationAction::Passthrough { name } => {
          if let Some(header) = ctx.downstream_http_request.headers.get(name) {
            match HeaderName::from_str(name) {
              Ok(header_name) => {
                req.headers.insert(header_name, header.clone());
              }
              Err(_) => {
                tracing::warn!(
                  "Plugin header_manipulation failed to parse header name '{:?}'",
                  name
                );
              }
            }
          }
        }
        HeaderManipulationAction::Remove { name } => {
          req.headers.remove(name);
        }
        HeaderManipulationAction::Add { name, value } => {
          match (HeaderName::from_str(name), HeaderValue::from_str(value)) {
            (Ok(header_name), Ok(header_value)) => {
              req.headers.insert(header_name, header_value);
            }
            _ => {
              tracing::warn!(
                "Plugin header_manipulation failed to parse header name or value from '{:?}'",
                name
              );
            }
          }
        }
      }
    }
  }
}
