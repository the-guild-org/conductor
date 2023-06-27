use std::{
    convert::Infallible,
    pin::Pin,
};

use crate::config::EndpointDefinition;
use async_graphql_axum::GraphQLRequest;
use hyper::{service::Service, Body};
use axum::http::Request;

pub type EndpointRequest = Request<Body>;
pub type EndpointResponse = hyper::Response<Body>;
pub type EndpointError = Infallible;

pub type EndpointFuture = Pin<
    Box<
        (dyn std::future::Future<Output = Result<EndpointResponse, EndpointError>>
             + std::marker::Send
             + 'static),
    >,
>;

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
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: EndpointRequest) -> Self::Future {
        println!("call is called, req: {:?}", req);
        // This is probably where some aspects of the plugins should go
        // Some plugins will need to run before we even parse the incoming request: cache (based on HTTP caching), persisted operations and so on.
        // In the meantime, we'll try to establish a loose contract between this service and the upstream service, by only doing GraphQL validations and pass a
        // GraphQLRequest forward.
        // This means we also expect to get a GraphQLResponse from the upstream service, which we'll then transform into a HTTP response here.
        // hyper::Request -> GraphQLRequest -> GraphQLResponse -> hyper::Response

        todo!()
    }
}
