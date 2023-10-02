pub mod config;
pub mod endpoint;
pub mod graphql_utils;
pub mod http_utils;
pub mod plugins;
pub mod source;
pub mod test;

use std::sync::Arc;

use axum::{Extension, Router, Server};
use axum_macros::debug_handler;
use config::{load_config, ConductorConfig};
use endpoint::endpoint_runtime::EndpointRuntime;

use axum::http::Request;
use axum::response::IntoResponse;
use axum::routing::{any, IntoMakeService};
use graphql_utils::GraphQLResponse;
use http::StatusCode;
use hyper::Body;

use plugins::flow_context::FlowContext;
use tracing::{debug, info};

use crate::config::SourceDefinition;
use crate::graphql_utils::ParsedGraphQLRequest;
use crate::http_utils::extract_graphql_from_post_request;
use crate::plugins::plugin_manager::PluginManager;
use crate::source::graphql_source::GraphQLSourceService;

pub struct RouterState {
    pub plugin_manager: Arc<PluginManager>,
}

#[debug_handler]
pub async fn http_request_handler(
    Extension(endpoint): Extension<EndpointRuntime>,
    mut request: Request<Body>,
) -> impl IntoResponse {
    // This represents the main context that is shared across the execution.
    // We'll use this struct to pass data between the plugins, endpoint and source.
    let mut flow_ctx = FlowContext::new(&endpoint, &mut request);

    // Execute plugins on the HTTP request.
    // Later we need to think how to expose things like the request body to the
    // plugins, if needed,without having to read it twice.
    endpoint
        .plugin_manager
        .on_downstream_http_request(&mut flow_ctx)
        .await;

    // In case the response was set by one of the plugins at this stage, just short-circuit and return it.
    if flow_ctx.short_circuit_response.is_some() {
        let mut sc_response = flow_ctx.short_circuit_response.unwrap();
        flow_ctx.short_circuit_response = None;
        endpoint
            .plugin_manager
            .on_downstream_http_response(&flow_ctx, &mut sc_response);

        return sc_response.into_response();
    }

    // If we can't extract anything from the request, we can try to do that here.
    // Plugins might have set it before, so we can avoid extraction.
    if flow_ctx.downstream_graphql_request.is_none()
        && flow_ctx.downstream_http_request.method() == axum::http::Method::POST
    {
        debug!("captured POST request, trying to handle as GraphQL POST flow");
        let (_, accept, result) = extract_graphql_from_post_request(&mut flow_ctx).await;

        match result {
            Ok(gql_request) => match ParsedGraphQLRequest::create_and_parse(gql_request) {
                Ok(parsed) => {
                    flow_ctx.downstream_graphql_request = Some(parsed);
                }
                Err(e) => {
                    return e.into_response(accept);
                }
            },
            Err(e) => {
                debug!(
                    "error while trying to extract GraphQL request from POST request: {:?}",
                    e
                );

                return e.into_response(accept);
            }
        }
    }

    if flow_ctx.has_failed_extraction() {
        return GraphQLResponse::new_error("failed to extract GraphQL request from HTTP request")
            .into_response(StatusCode::BAD_REQUEST);
    }

    // Execute plugins on the GraphQL request.
    endpoint
        .plugin_manager
        .on_downstream_graphql_request(&mut flow_ctx)
        .await;

    // In case the response was set by one of the plugins at this stage, just short-circuit and return it.
    if flow_ctx.short_circuit_response.is_some() {
        let mut sc_response = flow_ctx.short_circuit_response.unwrap();
        flow_ctx.short_circuit_response = None;
        endpoint
            .plugin_manager
            .on_downstream_http_response(&flow_ctx, &mut sc_response);

        return sc_response.into_response();
    }

    // Run the actual endpoint handler and get the response.
    let (flow_ctx, endpoint_response) = endpoint.handle_request(flow_ctx).await;

    let mut final_response = match endpoint_response {
        Ok(response) => response.into_response(),
        Err(e) => e.into_response(),
    };

    endpoint
        .plugin_manager
        .on_downstream_http_response(&flow_ctx, &mut final_response);

    final_response
}

pub(crate) fn create_router_from_config(config_object: ConductorConfig) -> IntoMakeService<Router> {
    tracing_subscriber::fmt()
        .with_max_level(config_object.logger.level.into_level())
        .init();

    debug!("loaded gateway config: {:?}", config_object);
    let mut http_router = Router::new();

    let global_plugins = &config_object.plugins;
    debug!("global plugins configured: {:?}", global_plugins);

    for endpoint_config in config_object.endpoints.into_iter() {
        info!("declaring endpoint on route {:?}", endpoint_config.path);

        let combined_plugins = global_plugins
            .iter()
            .chain(&endpoint_config.plugins)
            .flat_map(|vec| vec.iter())
            .cloned()
            .collect::<Vec<_>>();

        info!("adding plugins to route {:?}", combined_plugins);
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

        let endpoint_runtime = EndpointRuntime::new(
            endpoint_config.clone(),
            upstream_source,
            plugin_manager.clone(),
        );

        debug!("creating router route");

        http_router = http_router
            .route(endpoint_config.path.as_str(), any(http_request_handler))
            .route_layer(Extension(endpoint_runtime));

        debug!("calling on_endpoint_creation on route");
        http_router = plugin_manager.on_endpoint_creation(http_router);
    }

    http_router.into_make_service()
}

pub async fn run_services(config_file_path: String) {
    println!("gateway process started");
    println!("loading configuration from {}", config_file_path);
    let config_object = load_config(&config_file_path).await;
    println!("configuration loaded");
    let server_config = config_object.server.clone();
    let router_service = create_router_from_config(config_object);
    let server_address = format!("{}:{}", server_config.host, server_config.port);
    debug!("server is trying to listen on {:?}", server_address);
    Server::bind(&server_address.as_str().parse().unwrap())
        .serve(router_service)
        .await
        .unwrap();
}
