use std::{pin::Pin, convert::Infallible, sync::{Arc, RwLock}};

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLResponse, GraphQLRequest};
use axum::{routing::MethodRouter, response::{self, IntoResponse}, Extension, extract::State};
use hyper::{service::Service, Request, Body};
use axum::{routing::get
};
use crate::config::EndpointDefinition;

pub type EndpointRequest = hyper::Request<Body>;
pub type EndpointResponse = GraphQLResponse;
pub type EndpointError = Infallible;

pub type EndpointFuture = Pin<
    Box<
        (dyn std::future::Future<Output = Result<EndpointResponse, EndpointError>>
             + std::marker::Send
             + 'static),
    >,
>;

async fn graphiql(req: Request<Body>) -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint(req.uri().path()).finish())
}

#[derive(Clone)]
pub struct EndpointRuntime {
    pub config: EndpointDefinition,
}

pub type SharedState = Arc<RwLock<EndpointRuntime>>;

async fn post_handler(req: GraphQLRequest) -> GraphQLResponse {
    println!("post_handler");

    // endpoint_runtime.call(req)
    todo!()
}

impl EndpointRuntime {
    pub fn new(config: EndpointDefinition) -> Self {
        Self { config }
    }
}

impl Service<EndpointRequest> for EndpointRuntime {
    type Response = EndpointResponse;
    type Error = EndpointError;
    type Future = EndpointFuture;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        todo!()
    }

    fn call(&mut self, req: EndpointRequest) -> Self::Future {
        todo!()
    }
}