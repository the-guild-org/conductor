pub mod config;
pub mod endpoint;
pub mod gateway;
pub mod source;

use async_graphql::http::GraphiQLSource;
use axum::extract::State;
use axum::{Router, Server};
use config::load_config;
use endpoint::endpoint_runtime::EndpointRuntime;
use gateway::engine::Gateway;

use axum::http::Request;
use axum::response::{self, IntoResponse, Response};
use axum::routing::get;
use hyper::{Body, Client};

use tracing::debug;

use crate::config::{IntrospectionConfig, SourceDefinition};
use crate::source::introspection;

pub async fn serve_graphiql_ide(req: Request<Body>) -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint(req.uri().path()).finish())
}

pub async fn handle_post(State(state): State<EndpointRuntime>, body: String) -> Response<Body> {
    let response = state.call(body).await;
    response.unwrap()
}

pub async fn run_services(config_file_path: String) {
    println!("gateway process started");
    println!("loading configuration from {}", config_file_path);
    let config = load_config(&config_file_path).await;
    println!("configuration loaded");

    // Create a new Hyper client
    let http_client = Client::new();

    // Loop over all sources in the config and start introspection tasks
    for source in config.sources.clone() {
        match source {
            SourceDefinition::GraphQL { id, config } => {
                if let Some(introspection) = config.introspection.clone() {
                    match introspection {
                        IntrospectionConfig::source {
                            headers,
                            polling_interval,
                        } => {
                            // Do the source introspection for this id
                            // You can use headers and polling_interval if they are Some
                            tokio::spawn(introspection::fetch::fetch_from_source(
                                config.endpoint.clone(),
                                headers,
                                // polling_interval,
                            ));
                        }
                        IntrospectionConfig::json { location } => {
                            // Do the JSON introspection for this id
                            // You have the location which can be a local file path or a URL
                            // tokio::spawn(introspection::fetch::fetch_from_json(
                            //     &http_client,
                            //     // &config,
                            //     // location,
                            // ));
                        }
                    }
                } else {
                    // If no introspection config is provided for this id, just establish a connection
                    // Replace the following line with your actual function to establish connection
                    // establish_connection(&client, &config)
                }
            }
            // Other types of sources would be handled here
            _ => {}
        }
    }

    tracing_subscriber::fmt()
        .with_max_level(config.logger.level.into_level())
        .init();

    debug!("loaded gateway config: {:?}", config);

    let gateway = Gateway::new(config);
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
