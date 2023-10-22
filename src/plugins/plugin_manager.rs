use axum::{body::BoxBody, Router};
use hyper::Body;

use crate::{
    config::PluginDefinition, endpoint::endpoint_runtime::EndpointError,
    graphql_utils::GraphQLRequest, plugins::core::Plugin,
};

use super::{
    cors::CorsPlugin, flow_context::FlowContext, graphiql_plugin::GraphiQLPlugin,
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
                PluginDefinition::CorsPlugin { config } => {
                    instance.register_plugin(CorsPlugin(config.clone()))
                }
                PluginDefinition::GraphiQLPlugin => instance.register_plugin(GraphiQLPlugin {}),
                PluginDefinition::HttpGetPlugin { config } => {
                    instance.register_plugin(HttpGetPlugin(config.clone().unwrap_or_default()))
                }
                PluginDefinition::PersistedOperationsPlugin { config } => instance.register_plugin(
                    PersistedOperationsPlugin::new_from_config(config.clone())
                        .expect("failed to initalize persisted operations plugin"),
                ),
            });
        }

        // We want to make sure to register this one last, in order to ensure it's setting the value correctly
        instance.register_plugin(MatchContentTypePlugin {});

        instance
    }

    pub fn register_plugin(&mut self, plugin: impl Plugin + 'static) {
        self.plugins.push(Box::new(plugin));
    }

    #[tracing::instrument(level = "trace")]
    pub async fn on_downstream_http_request(&self, context: &mut FlowContext<'_>) {
        let p = &self.plugins;

        for plugin in p.iter() {
            plugin.on_downstream_http_request(context).await;

            if context.is_short_circuit() {
                return;
            }
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn on_downstream_http_response(
        &self,
        context: &FlowContext<'_>,
        response: &mut http::Response<BoxBody>,
    ) {
        let p = &self.plugins;

        for plugin in p.iter() {
            plugin.on_downstream_http_response(context, response);

            if context.is_short_circuit() {
                return;
            }
        }
    }

    #[tracing::instrument(level = "trace")]
    pub async fn on_downstream_graphql_request(&self, context: &mut FlowContext<'_>) {
        let p = &self.plugins;

        for plugin in p.iter() {
            plugin.on_downstream_graphql_request(context).await;

            if context.is_short_circuit() {
                return;
            }
        }
    }

    #[tracing::instrument(level = "trace")]
    pub async fn on_upstream_graphql_request<'a>(&self, req: &mut GraphQLRequest) {
        let p = &self.plugins;

        for plugin in p.iter() {
            plugin.on_upstream_graphql_request(req).await;
        }
    }

    #[tracing::instrument(level = "trace")]
    pub async fn on_upstream_graphql_response<'a>(
        &self,
        response: &mut Result<hyper::Response<Body>, EndpointError>,
    ) {
        let p = &self.plugins;

        for plugin in p.iter() {
            plugin.on_upstream_graphql_response(response).await;
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn on_endpoint_creation<'a>(
        &self,
        root_router: Router<()>,
        endpoint_router: Router<()>,
    ) -> (Router<()>, Router<()>) {
        let p: &Vec<Box<dyn Plugin>> = &self.plugins;
        let mut modified_root_router = root_router;
        let mut modified_endpoint_router = endpoint_router;

        for plugin in p.iter() {
            (modified_root_router, modified_endpoint_router) =
                plugin.on_endpoint_creation(modified_root_router, modified_endpoint_router);
        }

        (modified_root_router, modified_endpoint_router)
    }
}
