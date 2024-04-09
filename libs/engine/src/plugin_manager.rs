use std::sync::Arc;

use conductor_common::{
  execute::RequestExecutionContext,
  graphql::GraphQLRequest,
  http::{ConductorHttpRequest, ConductorHttpResponse},
  logging_locks::LoggingRwLock,
  plugin::{CreatablePlugin, Plugin, PluginError},
  plugin_manager::PluginManager,
  source::SourceRuntime,
};
use conductor_config::PluginDefinition;
use conductor_tracing::minitrace_mgr::MinitraceManager;
use no_deadlocks::RwLock;
use reqwest::Response;

#[derive(Debug, Default)]
pub struct PluginManagerImpl {
  plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManagerImpl {
  pub fn new_from_vec(plugins: Vec<Box<dyn Plugin>>) -> Self {
    let mut pm = Self { plugins };

    // We want to make sure to register default plugins last, in order to ensure it's setting the value correctly
    for p in PluginManagerImpl::default_plugins() {
      pm.register_boxed_plugin(p);
    }

    pm
  }

  pub async fn create_plugin<T: CreatablePlugin>(config: T::Config) -> Result<Box<T>, PluginError> {
    T::create(config).await
  }

  pub async fn new(
    plugins_config: &Option<Vec<PluginDefinition>>,
    tracing_manager: &mut MinitraceManager,
    tenant_id: u32,
  ) -> Result<Self, PluginError> {
    let mut instance = PluginManagerImpl::default();

    if let Some(config_defs) = plugins_config {
      for plugin_def in config_defs.iter() {
        let plugin: Box<dyn Plugin> = match plugin_def {
          PluginDefinition::GraphiQLPlugin {
            enabled: Some(true),
            config,
          } => {
            Self::create_plugin::<graphiql_plugin::Plugin>(config.clone().unwrap_or_default())
              .await?
          }
          PluginDefinition::HttpGetPlugin {
            enabled: Some(true),
            config,
          } => {
            Self::create_plugin::<http_get_plugin::Plugin>(config.clone().unwrap_or_default())
              .await?
          }
          PluginDefinition::VrlPluginConfig {
            enabled: Some(true),
            config,
          } => Self::create_plugin::<vrl_plugin::Plugin>(config.clone()).await?,
          PluginDefinition::TrustedDocumentsPlugin {
            enabled: Some(true),
            config,
          } => Self::create_plugin::<trusted_documents_plugin::Plugin>(config.clone()).await?,
          PluginDefinition::CorsPlugin {
            enabled: Some(true),
            config,
          } => {
            Self::create_plugin::<cors_plugin::Plugin>(config.clone().unwrap_or_default()).await?
          }
          PluginDefinition::DisableItrospectionPlugin {
            enabled: Some(true),
            config,
          } => {
            Self::create_plugin::<disable_introspection_plugin::Plugin>(
              config.clone().unwrap_or_default(),
            )
            .await?
          }
          PluginDefinition::JwtAuthPlugin {
            enabled: Some(true),
            config,
          } => Self::create_plugin::<jwt_auth_plugin::Plugin>(config.clone()).await?,
          PluginDefinition::GraphQLValidation {
            enabled: Some(true),
            config,
          } => {
            Self::create_plugin::<graphql_validation_plugin::Plugin>(
              config.clone().unwrap_or_default(),
            )
            .await?
          }
          PluginDefinition::TelemetryPlugin {
            enabled: Some(true),
            config,
          } => {
            let plugin = Self::create_plugin::<telemetry_plugin::Plugin>(config.clone()).await?;
            plugin.configure_tracing(tenant_id, tracing_manager)?;

            plugin
          }
          // In case plugin is not enabled, we are skipping it. Also when we don't have a match, so watch out for this one if you add a new plugin.
          _ => continue,
        };

        instance.register_boxed_plugin(plugin)
      }
    };

    // We want to make sure to register these last, in order to ensure it's setting the value correctly
    for p in PluginManagerImpl::default_plugins() {
      instance.register_boxed_plugin(p);
    }

    Ok(instance)
  }

  fn default_plugins() -> Vec<Box<dyn Plugin>> {
    vec![Box::new(match_content_type_plugin::Plugin {})]
  }

  pub fn register_boxed_plugin(&mut self, plugin: Box<dyn Plugin>) {
    self.plugins.push(plugin);
  }

  pub fn register_plugin(&mut self, plugin: impl Plugin + 'static) {
    self.plugins.push(Box::new(plugin));
  }
}

#[async_trait::async_trait(?Send)]
impl PluginManager for PluginManagerImpl {
  #[tracing::instrument(
    level = "debug",
    skip(self, context),
    name = "on_downstream_http_request"
  )]
  #[inline]
  async fn on_downstream_http_request(&self, context: Arc<RwLock<RequestExecutionContext>>) {
    let p = &self.plugins;

    for plugin in p.iter() {
      plugin.on_downstream_http_request(context.clone()).await;

      if context.read().unwrap().is_short_circuit() {
        return;
      }
    }
  }

  #[tracing::instrument(
    level = "debug",
    skip(self, context, response),
    name = "on_downstream_http_response"
  )]
  #[inline]
  async fn on_downstream_http_response(
    &self,
    context: Arc<RwLock<RequestExecutionContext>>,
    response: &mut ConductorHttpResponse,
  ) {
    let p = &self.plugins;

    let should_short_circuit = {
      let ctx = context.read();

      ctx.unwrap().is_short_circuit()
    };

    if should_short_circuit {
      return;
    }

    for plugin in p.iter() {
      plugin
        .on_downstream_http_response(context.clone(), response)
        .await;

      let is_short_circuit_now = {
        let ctx = context.write();
        ctx.unwrap().is_short_circuit()
      };

      if is_short_circuit_now {
        return;
      }
    }
  }
  #[tracing::instrument(
    level = "debug",
    skip(self, context),
    name = "on_downstream_graphql_request"
  )]
  #[inline]
  async fn on_downstream_graphql_request(
    &self,
    source_runtime: Arc<Box<dyn SourceRuntime>>,
    context: Arc<RwLock<RequestExecutionContext>>,
  ) {
    let p = &self.plugins;

    for plugin in p.iter() {
      plugin
        .on_downstream_graphql_request(source_runtime.clone(), context.clone())
        .await;

      if context.read().unwrap().is_short_circuit() {
        return;
      }
    }
  }

  #[tracing::instrument(level = "debug", skip(self, req), name = "on_upstream_graphql_request")]
  #[inline]
  async fn on_upstream_graphql_request(&self, req: &mut GraphQLRequest) {
    let p = &self.plugins;

    for plugin in p.iter() {
      plugin.on_upstream_graphql_request(req).await;
    }
  }

  #[tracing::instrument(
    level = "debug",
    skip(self, ctx, request),
    name = "on_upstream_http_request"
  )]
  #[inline]
  async fn on_upstream_http_request(
    &self,
    ctx: Arc<RwLock<RequestExecutionContext>>,
    request: &mut ConductorHttpRequest,
  ) {
    let p = &self.plugins;

    for plugin in p.iter() {
      plugin.on_upstream_http_request(ctx.clone(), request).await;

      if ctx.write().unwrap().is_short_circuit() {
        return;
      }
    }
  }

  #[tracing::instrument(
    level = "debug",
    skip(self, ctx, response),
    name = "on_upstream_http_response"
  )]
  #[inline]
  async fn on_upstream_http_response(
    &self,
    ctx: Arc<RwLock<RequestExecutionContext>>,
    response: &Result<Response, reqwest_middleware::Error>,
  ) {
    let p = &self.plugins;

    for plugin in p.iter() {
      plugin
        .on_upstream_http_response(ctx.clone(), response)
        .await;

      if ctx.write().unwrap().is_short_circuit() {
        return;
      }
    }
  }
}
