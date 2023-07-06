pub mod config;
pub mod endpoint;
pub mod gateway;
pub mod source;

use async_graphql::http::GraphiQLSource;
use axum::extract::State;
use axum::{Router, Server};
use config::load_config;
use endpoint::endpoint_runtime::{EndpointRuntime, OnRequestPlugin, OnResponsePlugin};
use gateway::engine::Gateway;

use axum::response::{self, IntoResponse, Response};
use axum::routing::get;
use hyper::{Body, Request};

use tracing::debug;

pub async fn serve_graphiql_ide(req: Request<Body>) -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint(req.uri().path()).finish())
}

pub struct PluginsManager {
    on_request_plugins: Option<Vec<Box<dyn OnRequestPlugin>>>,
    on_response_plugins: Option<Vec<Box<dyn OnResponsePlugin>>>,
    on_cache_retrieval_plugin: Option<Box<dyn OnResponsePlugin>>,
}

impl PluginsManager {
    fn get_plugins(&self) -> Vec<Box<dyn OnRequestPlugin>> {
        // TODO: implement
    }

    fn add_on_request_plugin(&self) -> () {}
    fn add_on_response_plugins(&self) -> () {}
    fn add_on_cache_retrieval_plugin(&self) -> () {}

    // and other useful utilities
}

pub async fn handle_post(State(state): State<EndpointRuntime>, body: String) -> Response<Body> {
    let response = state.call(body).await;
    response.unwrap()
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

    let plugins_manager: PluginsManager = PluginsManager {
        on_request_plugins: Some(vec![]),
        on_response_plugins: Some(vec![]),
        on_cache_retrieval_plugin: None,
    };
    let gateway = Gateway::new(config_object, plugins_manager);
    let mut http_router = Router::new();

    for (path, endpoint) in gateway.endpoints.into_iter() {
        let path_str = path.as_str();
        http_router = http_router.route(
            path_str,
            get(serve_graphiql_ide)
                .post(handle_post)
                .with_state(endpoint),
        )
    }

    println!("GraphiQL IDE: http://localhost:8000");

    Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(http_router.into_make_service())
        .await
        .unwrap();
}
