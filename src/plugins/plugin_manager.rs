use axum::Router;
use hyper::Body;

use crate::{
    config::PluginDefinition, endpoint::endpoint_runtime::EndpointError, plugins::core::Plugin,
    source::base_source::SourceRequest,
};

use super::{
    cors::CorsPlugin, flow_context::FlowContext,
    json_content_type_response_plugin::JSONContentTypePlugin,
    verbose_logging_plugin::VerboseLoggingPlugin,
};

#[derive(Debug)]
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        PluginManager {
            plugins: vec![Box::new(JSONContentTypePlugin {})],
        }
    }
}

impl PluginManager {
    pub fn new(plugins_config: &Option<Vec<PluginDefinition>>) -> Self {
        let mut instance = PluginManager::default();

        if let Some(config_defs) = plugins_config {
            config_defs.iter().for_each(|plugin_def| match plugin_def {
                PluginDefinition::VerboseLogging => {
                    instance.register_plugin(VerboseLoggingPlugin {})
                }
                PluginDefinition::JSONContentTypeResponse => {
                    instance.register_plugin(JSONContentTypePlugin {})
                }
                PluginDefinition::CorsPlugin(config) => {
                    instance.register_plugin(CorsPlugin(config.clone()))
                }
            });
        }

        instance
    }

    pub fn register_plugin(&mut self, plugin: impl Plugin + 'static) {
        self.plugins.push(Box::new(plugin));
    }

    #[tracing::instrument(level = "trace")]
    pub fn on_downstream_http_request(&self, context: &mut FlowContext) {
        let p = &self.plugins;

        for plugin in p.iter() {
            plugin.on_downstream_http_request(context);

            if context.short_circuit_response.is_some() {
                return;
            }
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn on_downstream_http_response(&self, context: &mut FlowContext) {
        let p = &self.plugins;

        for plugin in p.iter() {
            plugin.on_downstream_http_response(context);

            if context.short_circuit_response.is_some() {
                return;
            }
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn on_downstream_graphql_request(&self, context: &mut FlowContext) {
        let p = &self.plugins;

        for plugin in p.iter() {
            plugin.on_downstream_graphql_request(context);

            if context.short_circuit_response.is_some() {
                return;
            }
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn on_upstream_graphql_request<'a>(&self, req: &mut SourceRequest<'a>) {
        let p = &self.plugins;

        for plugin in p.iter() {
            plugin.on_upstream_graphql_request(req);
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn on_upstream_graphql_response<'a>(
        &self,
        response: &mut Result<hyper::Response<Body>, EndpointError>,
    ) {
        let p = &self.plugins;

        for plugin in p.iter() {
            plugin.on_upstream_graphql_response(response);
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn on_endpoint_creation<'a>(&self, router: Router<()>) -> Router<()> {
        let p = &self.plugins;
        let mut modified_router = router;

        for plugin in p.iter() {
            modified_router = plugin.on_endpoint_creation(modified_router);
        }

        modified_router
    }
}
