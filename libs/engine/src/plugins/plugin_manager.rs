use conductor_common::{
    graphql::GraphQLRequest,
    http::{ConductorHttpRequest, ConductorHttpResponse},
};
use conductor_config::PluginDefinition;
use reqwest::{Error, Response};

use crate::request_execution_context::RequestExecutionContext;

use super::{
    core::Plugin, graphiql_plugin::GraphiQLPlugin, http_get_plugin::HttpGetPlugin,
    match_content_type::MatchContentTypePlugin,
    persisted_documents::plugin::PersistedOperationsPlugin, vrl::plugin::VrlPlugin,
};

#[derive(Debug, Default)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new_from_vec(plugins: Vec<Box<dyn Plugin>>) -> Self {
        let mut pm = Self { plugins };

        // We want to make sure to register this one last, in order to ensure it's setting the value correctly
        for p in PluginManager::default_plugins() {
            pm.register_boxed_plugin(p);
        }

        pm
    }

    pub fn new(plugins_config: &Option<Vec<PluginDefinition>>) -> Self {
        let mut instance = PluginManager::default();

        if let Some(config_defs) = plugins_config {
            config_defs.iter().for_each(|plugin_def| match plugin_def {
                PluginDefinition::GraphiQLPlugin { enabled, config } => {
                    if enabled.is_some_and(|v| v) {
                        instance.register_plugin(GraphiQLPlugin(config.clone().unwrap_or_default()))
                    }
                }
                PluginDefinition::HttpGetPlugin { enabled, config } => {
                    if enabled.is_some_and(|v| v) {
                        instance.register_plugin(HttpGetPlugin(config.clone().unwrap_or_default()))
                    }
                }
                PluginDefinition::VrlPluginConfig { enabled, config } => {
                    if enabled.is_some_and(|v| v) {
                        instance.register_plugin(VrlPlugin::new(config.clone()))
                    }
                }
                PluginDefinition::PersistedOperationsPlugin { enabled, config } => {
                    if enabled.is_some_and(|v| v) {
                        instance.register_plugin(
                            PersistedOperationsPlugin::new_from_config(config.clone())
                                .expect("failed to initalize persisted operations plugin"),
                        )
                    }
                }
            });
        }

        // We want to make sure to register this one last, in order to ensure it's setting the value correctly
        for p in PluginManager::default_plugins() {
            instance.register_boxed_plugin(p);
        }

        instance
    }

    fn default_plugins() -> Vec<Box<dyn Plugin>> {
        vec![Box::new(MatchContentTypePlugin {})]
    }

    pub fn register_boxed_plugin(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub fn register_plugin(&mut self, plugin: impl Plugin + 'static) {
        self.plugins.push(Box::new(plugin));
    }

    #[tracing::instrument(level = "debug", skip(self, context))]
    pub async fn on_downstream_http_request(&self, context: &mut RequestExecutionContext<'_>) {
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
    pub async fn on_downstream_graphql_request(&self, context: &mut RequestExecutionContext<'_>) {
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

    #[tracing::instrument(level = "debug", skip(self, request))]
    pub async fn on_upstream_http_request<'a>(
        &self,
        ctx: &mut RequestExecutionContext<'_>,
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

    #[tracing::instrument(level = "debug", skip(self, response))]
    pub async fn on_upstream_http_response<'a>(
        &self,
        ctx: &mut RequestExecutionContext<'_>,
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
