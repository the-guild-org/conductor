mod config;
mod gateway;
mod endpoint;
mod source;

use std::{convert::Infallible, sync::Arc};

use crate::config::load_config;
use crate::gateway::engine::Gateway;
use axum::{
    Router, Server, response::Response, Extension,
};
use async_graphql::http::GraphiQLSource;

use axum::{response::{self, IntoResponse}};
use endpoint::endpoint::EndpointRuntime;
use hyper::{service::{service_fn}, Request, Body};
use axum::{routing::get};
use tracing::debug;
use tracing_subscriber;

pub async fn graphiql(req: Request<Body>) -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint(req.uri().path()).finish())
}

pub async fn handle_post(req: Request<Body>, Extension(r): Extension<Arc<EndpointRuntime>>) -> impl IntoResponse {
    Ok::<_, Infallible>(Response::new(Body::empty()))
}

#[tokio::main]
async fn main() {
    println!("gateway process started");
    let config_file_path = std::env::args().nth(1).unwrap_or("./conductor.json".to_string());
    println!("loading configuration from {}", config_file_path);
    let config_object = load_config(&config_file_path).await;
    println!("configuration loaded");

    tracing_subscriber::fmt()
        .with_max_level(config_object.logger.level.into_level())
        .init();

    debug!("loaded gateway config: {:?}", config_object);

    let gateway = Gateway::new(config_object);
    let mut http_router = Router::new();

    let service = service_fn(|request: Request<Body>, | async {
    });

    for (path, endpoint) in gateway.endpoints.into_iter() {
        http_router = http_router.route(path.as_str(), get(graphiql).post(handle_post).layer(Extension(Arc::new(endpoint))));
    }

    // gateway.declare_endpoints_as_routes(&mut http_router);

    // let app = Router::new()
    //     .route("/a", get(graphiql).post(graphql_handler::<Schema>))
    //     .route("/b", get(graphiql).post(graphql_handler::<Schema>))
    //     .layer(Extension(schema));

    println!("GraphiQL IDE: http://localhost:8000");

    Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(http_router.into_make_service())
        .await
        .unwrap();
}
