mod config;
mod endpoint;
mod gateway;
mod source;

use crate::config::load_config;
use crate::gateway::engine::Gateway;
use async_graphql::http::GraphiQLSource;
use axum::{Router, Server};

use axum::http::Request;
use axum::response::Response;
use axum::response::{self, IntoResponse};
use axum::routing::get;
use hyper::service::Service;
use hyper::Body;
use hyper::{header, StatusCode};
use std::sync::RwLock;
use std::{convert::Infallible, sync::Arc};

use axum::response::Result;
use endpoint::endpoint::{EndpointError, EndpointRequest, EndpointResponse, EndpointRuntime};
use tracing::debug;
use tracing_subscriber;

pub async fn graphiql(req: Request<Body>) -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint(req.uri().path()).finish())
}

pub async fn run_endpoint_runtime(
    req: EndpointRequest,
    runtime: &mut EndpointRuntime,
) -> Result<EndpointResponse, EndpointError> {
    // Call the service with the request
    runtime.call(req).await
}

pub async fn handle_post(req: Request<Body>) -> Result<impl IntoResponse> {
    let extension: Option<Arc<RwLock<EndpointRuntime>>> = req
        .extensions()
        .get::<Arc<RwLock<EndpointRuntime>>>()
        .cloned();

    println!("{:?}", extension);

    // return Ok::<_, Infallible>(Response::new("hello world".to_string()));
    match extension {
        Some(runtime) => {
            let (parts, body) = req.into_parts();
            // Convert the Axum Request into an EndpointRequest
            let endpoint_req = EndpointRequest::from_parts(parts, body);

            // Lock and obtain mutable access
            let mut runtime = runtime.write().unwrap();

            // Call the service with the request
            match runtime.call(endpoint_req).await {
                Ok(response) => Ok(response),
                Err(_) => {
                    // If there was an error, return a 500 response
                    let response = Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::from("Internal server error"))
                        .unwrap();

                    Ok(response)
                }
            }
        }
        None => {
            // If there was no EndpointRuntime in the extensions, return a 500 response
            let response = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"error": "Missing runtime"}"#))
                .unwrap();

            Ok::<_, Infallible>(response.into_response())
        }
    }
}

#[tokio::main]
async fn main() {
    println!("gateway process started");
    let config_file_path = std::env::args()
        .nth(1)
        .unwrap_or("./conductor.json".to_string());
    println!("loading configuration from {}", config_file_path);
    let config_object = load_config(&config_file_path).await;
    println!("configuration loaded");

    tracing_subscriber::fmt()
        .with_max_level(config_object.logger.level.into_level())
        .init();

    debug!("loaded gateway config: {:?}", config_object);

    let gateway = Gateway::new(config_object);
    let mut http_router = Router::new();

    for (path, endpoint) in gateway.endpoints.into_iter() {
        http_router = http_router.route(path.as_str(), get(graphiql).post_service(endpoint));
    }

    println!("GraphiQL IDE: http://localhost:8000");

    Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(http_router.into_make_service())
        .await
        .unwrap();
}
