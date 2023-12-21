use conductor_common::{
  execute::RequestExecutionContext,
  graphql::GraphQLRequest,
  http::{ConductorHttpRequest, ConductorHttpResponse},
  plugin::{CreatablePlugin, Plugin, PluginError},
};
use conductor_config::PluginDefinition;
use reqwest::{Error, Response};
#[derive(Debug, Default)]
pub struct PluginManager {
  plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
  pub fn new_from_vec(plugins: Vec<Box<dyn Plugin>>) -> Self {
    let mut pm = Self { plugins };

    // We want to make sure to register default plugins last, in order to ensure it's setting the value correctly
    for p in PluginManager::default_plugins() {
      pm.register_boxed_plugin(p);
    }

    pm
  }

  pub async fn create_plugin<T: CreatablePlugin>(
    config: T::Config,
  ) -> Result<Box<dyn Plugin>, PluginError> {
    T::create(config).await
  }

  pub async fn new(plugins_config: &Option<Vec<PluginDefinition>>) -> Result<Self, PluginError> {
    let mut instance = PluginManager::default();

    if let Some(config_defs) = plugins_config {
      for plugin_def in config_defs.iter() {
        let plugin = match plugin_def {
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
          PluginDefinition::PersistedOperationsPlugin {
            enabled: Some(true),
            config,
          } => Self::create_plugin::<persisted_documents_plugin::Plugin>(config.clone()).await?,
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
          // In case plugin is not enabled, we are skipping it. Also when we don't have a match, so watch out for this one if you add a new plugin.
          _ => continue,
        };

        instance.register_boxed_plugin(plugin)
      }
    };

    // We want to make sure to register this one last, in order to ensure it's setting the value correctly
    for p in PluginManager::default_plugins() {
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

  #[tracing::instrument(level = "debug", skip(self, context))]
  pub async fn on_downstream_http_request(&self, context: &mut RequestExecutionContext) {
    let p = &self.plugins;

    for plugin in p.iter() {
      plugin.on_downstream_http_request(context).await;

      if context.is_short_circuit() {
        return;
      }
    }
  }

  #[tracing::instrument(level = "debug", skip(self, context, response))]
  pub fn on_downstream_http_response(
    &self,
    context: &mut RequestExecutionContext,
    response: &mut ConductorHttpResponse,
  ) {
    let p = &self.plugins;

    for plugin in p.iter() {
      plugin.on_downstream_http_response(context, response);

      if context.is_short_circuit() {
        return;
      }
    }
  }

  #[tracing::instrument(level = "debug", skip(self, context))]
  pub async fn on_downstream_graphql_request(&self, context: &mut RequestExecutionContext) {
    let p = &self.plugins;

    for plugin in p.iter() {
      plugin.on_downstream_graphql_request(context).await;

      if context.is_short_circuit() {
        return;
      }
    }
  }

  #[tracing::instrument(level = "debug", skip(self, req))]
  pub async fn on_upstream_graphql_request<'a>(&self, req: &mut GraphQLRequest) {
    let p = &self.plugins;

    for plugin in p.iter() {
      plugin.on_upstream_graphql_request(req).await;
    }
  }

  #[tracing::instrument(level = "debug", skip(self, ctx, request))]
  pub async fn on_upstream_http_request<'a>(
    &self,
    ctx: &mut RequestExecutionContext,
    request: &mut ConductorHttpRequest,
  ) {
    let p = &self.plugins;

    for plugin in p.iter() {
      plugin.on_upstream_http_request(ctx, request).await;

      if ctx.is_short_circuit() {
        return;
      }
    }
  }

  #[tracing::instrument(level = "debug", skip(self, ctx, response))]
  pub async fn on_upstream_http_response<'a>(
    &self,
    ctx: &mut RequestExecutionContext,
    response: &Result<Response, Error>,
  ) {
    let p = &self.plugins;

    for plugin in p.iter() {
      plugin.on_upstream_http_response(ctx, response).await;

      if ctx.is_short_circuit() {
        return;
      }
    }
  }
}
