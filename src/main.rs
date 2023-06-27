mod config;
mod endpoint;
mod gateway;
mod source;

use crate::config::load_config;
use crate::gateway::engine::Gateway;
use async_graphql::http::GraphiQLSource;
use axum::{Router, Server};

use axum::http::Request;
use axum::response::{self, IntoResponse};
use axum::routing::get;
use hyper::Body;
use tracing::debug;
use tracing_subscriber;

pub async fn graphiql(req: Request<Body>) -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint(req.uri().path()).finish())
}

// pub async fn handle_post(
//     State(endpoint_runtime): State<Arc<EndpointRuntime>>,
//     req: Request<Body>,
// ) -> impl IntoResponse {
//     endpoint_runtime.call(req);
//     println!("handle_post, endpoint_runtime: {:?}, req: {:?}", endpoint_runtime, req);

//     // TODO: Run the actual flow from EndpointRuntime
//     Ok::<_, Infallible>(Response::new(Body::empty()))
// }

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
