use std::{convert::Infallible, pin::Pin};

use crate::config::EndpointDefinition;
use hyper::{service::Service, Body};

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQLResponse;
use axum::response::{self, IntoResponse};
use hyper::{Request, Response};

pub type EndpointRequest = hyper::Request<Body>;
pub type EndpointResponse = hyper::Response<Body>;
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

#[derive(Clone, Debug)]
pub struct EndpointRuntime {
    pub config: EndpointDefinition,
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

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        // Check if the service is ready to handle requests
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: EndpointRequest) -> Self::Future {
        println!("call is called, req: {:?}", req);
        todo!()
    }
}
