use std::sync::{Arc, RwLock};

use crate::{config::PluginDefinition, plugins::core::Plugin, source::base_source::SourceRequest};

use super::{flow_context::FlowContext, verbose_logging_plugin::VerboseLoggingPlugin};

#[derive(Clone, Debug)]
pub struct PluginManager {
    plugins: Arc<RwLock<Vec<Box<dyn Plugin>>>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        PluginManager::new(&None)
    }
}

impl PluginManager {
    pub fn new(plugins_config: &Option<Vec<PluginDefinition>>) -> Self {
        let mut instance = Self {
            plugins: Arc::new(RwLock::new(Vec::new())),
        };

        if let Some(config_defs) = plugins_config {
            config_defs.iter().for_each(|plugin_def| match plugin_def {
                PluginDefinition::VerboseLogging => {
                    instance.register_plugin(VerboseLoggingPlugin {})
                }
            });
        }

        instance
    }

    pub fn register_plugin(&mut self, plugin: impl Plugin + 'static) {
        self.plugins.write().unwrap().push(Box::new(plugin));
    }

    #[tracing::instrument]
    pub fn on_downstream_http_request(&self, mut context: FlowContext) -> FlowContext {
        let p = self.plugins.read().unwrap();

        for plugin in p.iter() {
            context = plugin.on_downstream_http_request(context);

            if context.short_circuit_response.is_some() {
                return context;
            }
        }

        context
    }

    #[tracing::instrument]
    pub fn on_downstream_http_response(&self, mut context: FlowContext) -> FlowContext {
        let p = self.plugins.read().unwrap();

        for plugin in p.iter() {
            context = plugin.on_downstream_http_response(context);

            if context.short_circuit_response.is_some() {
                return context;
            }
        }

        context
    }

    #[tracing::instrument]
    pub fn on_downstream_graphql_request(&self, mut context: FlowContext) -> FlowContext {
        let p = self.plugins.read().unwrap();

        for plugin in p.iter() {
            context = plugin.on_downstream_graphql_request(context);

            if context.short_circuit_response.is_some() {
                return context;
            }
        }

        context
    }

    #[tracing::instrument]
    pub fn on_upstream_graphql_request(&self, mut req: SourceRequest) -> SourceRequest {
        let p = self.plugins.read().unwrap();

        for plugin in p.iter() {
            req = plugin.on_upstream_graphql_request(req);
        }

        req
    }
}
