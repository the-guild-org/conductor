pub mod config;
pub mod endpoint;
pub mod plugins;
pub mod source;

use std::sync::Arc;

use async_graphql::http::GraphiQLSource;
use axum::extract::BodyStream;
use axum::{Extension, Router, Server};
use axum_macros::debug_handler;
use config::load_config;
use endpoint::endpoint_runtime::EndpointRuntime;

use axum::http::Request;
use axum::response::{self, IntoResponse};
use axum::routing::get;
use hyper::{Body, HeaderMap};

use plugins::flow_context::FlowContext;
use tracing::debug;

use crate::config::SourceDefinition;
use crate::plugins::plugin_manager::PluginManager;
use crate::source::graphql_source::GraphQLSourceService;

pub struct RouterState {
    pub plugin_manager: Arc<PluginManager>,
}

pub async fn serve_graphiql_ide(req: Request<Body>) -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint(req.uri().path()).finish())
}

#[debug_handler]
pub async fn handle_post(
    Extension(endpoint): Extension<EndpointRuntime>,
    headers: HeaderMap,
    mut body_stream: BodyStream,
) -> impl IntoResponse {
    // This represents the main context that is shared across the execution.
    // We'll use this struct to pass data between the plugins, endpoint and source.
    let mut flow_ctx = FlowContext {
        downstream_graphql_request: None,
        short_circuit_response: None,
        downstream_headers: headers,
    };

    // Execute plugins on the HTTP request.
    // Later we need to think how to expose things like the request body to the
    // plugins, if needed,without having to read it twice.
    endpoint
        .plugin_manager
        .on_downstream_http_request(&mut flow_ctx);

    // In case the response was set by one of the plugins at this stage, just short-circuit and return it.
    if let Some(sc_response) = flow_ctx.short_circuit_response {
        return sc_response;
    }

    // In case the GraphQL operation was not extracted from the HTTP request yet, do it now.
    // This is done in order to allow plugins to override/set the GraphQL operation, for use-cases like persisted operations.
    if flow_ctx.downstream_graphql_request.is_none() {
        flow_ctx
            .extract_graphql_request_from_http_request(&mut body_stream)
            .await;
    }

    // Execute plugins on the GraphQL request.
    endpoint
        .plugin_manager
        .on_downstream_graphql_request(&mut flow_ctx);

    // In case the response was set by one of the plugins at this stage, just short-circuit and return it.
    if let Some(sc_response) = flow_ctx.short_circuit_response {
        return sc_response;
    }

    // Run the actual endpoint handler and get the response.
    let (mut flow_ctx, mut endpoint_response) = endpoint.handle_request(flow_ctx).await;

    endpoint
        .plugin_manager
        .on_downstream_http_response(&mut flow_ctx);

    endpoint
        .plugin_manager
        .on_upstream_graphql_response(&mut endpoint_response);

    match endpoint_response {
        Ok(response) => response,
        Err(e) => e.into(),
    }
}

pub async fn run_services(config_file_path: String) {
    println!("gateway process started");
    println!("loading configuration from {}", config_file_path);
    let config_object = load_config(&config_file_path).await;
    println!("configuration loaded");

    tracing_subscriber::fmt()
        .with_max_level(config_object.logger.level.into_level())
        .init();

    debug!("loaded gateway config: {:?}", config_object);

    let server_config = config_object.server.clone();
    let mut http_router = Router::new();

    let global_plugins = &config_object.plugins;

    for endpoint_config in config_object.endpoints.into_iter() {
        let combined_plugins = global_plugins
            .iter()
            .chain(&endpoint_config.plugins)
            .flat_map(|vec| vec.iter())
            .cloned()
            .collect::<Vec<_>>();

        let plugin_manager = Arc::new(PluginManager::new(&Some(combined_plugins)));

        let upstream_source = config_object
            .sources
            .iter()
            .find_map(|source_def| match source_def {
                SourceDefinition::GraphQL { id, config }
                    if id.eq(endpoint_config.from.as_str()) =>
                {
                    Some(GraphQLSourceService::from_config(
                        config.clone(),
                        plugin_manager.clone(),
                    ))
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("source with id {} not found", endpoint_config.from));

        let endpoint_runtime =
            EndpointRuntime::new(endpoint_config.clone(), upstream_source, plugin_manager);

        http_router = http_router
            .route(
                endpoint_config.path.as_str(),
                get(serve_graphiql_ide).post(handle_post),
            )
            .layer(Extension(endpoint_runtime));
    }

    let server_address = format!("{}:{}", server_config.host, server_config.port);
    Server::bind(&server_address.as_str().parse().unwrap())
        .serve(http_router.into_make_service())
        .await
        .unwrap();
}
