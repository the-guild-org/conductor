use conductor_common::{
    graphql::{GraphQLRequest, GraphQLResponse},
    http::ConductorHttpResponse,
};
use conductor_config::PluginDefinition;

use crate::request_execution_context::RequestExecutionContext;

use super::{
    context_building::ContextBuildingPlugin, core::Plugin, graphiql_plugin::GraphiQLPlugin,
    http_get_plugin::HttpGetPlugin, match_content_type::MatchContentTypePlugin,
    persisted_documents::plugin::PersistedOperationsPlugin,
};

#[derive(Debug, Default)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new(plugins_config: &Option<Vec<PluginDefinition>>) -> Self {
        let mut instance = PluginManager::default();

        if let Some(config_defs) = plugins_config {
            config_defs.iter().for_each(|plugin_def| match plugin_def {
                PluginDefinition::GraphiQLPlugin { enabled, config } => {
                    if enabled.is_some_and(|v| v) {
                        instance.register_plugin(GraphiQLPlugin(config.clone().unwrap_or_default()))
                    }
                }
                PluginDefinition::ContextBuilderPlugin { enabled, config } => {
                    if enabled.is_some_and(|v| v) {
                        instance.register_plugin(ContextBuildingPlugin(
                            config.clone().unwrap_or_default(),
                        ))
                    }
                }
                PluginDefinition::HttpGetPlugin { enabled, config } => {
                    if enabled.is_some_and(|v| v) {
                        instance.register_plugin(HttpGetPlugin(config.clone().unwrap_or_default()))
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
        instance.register_plugin(MatchContentTypePlugin {});

        instance
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
        context: &RequestExecutionContext,
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

    #[tracing::instrument(level = "debug", skip(self, response))]
    pub async fn on_upstream_graphql_response<'a>(&self, response: &mut GraphQLResponse) {
        let p = &self.plugins;

        for plugin in p.iter() {
            plugin.on_upstream_graphql_response(response).await;
        }
    }
}
