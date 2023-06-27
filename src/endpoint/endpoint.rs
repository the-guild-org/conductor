use std::{
    convert::Infallible,
    pin::Pin,
};

use crate::config::EndpointDefinition;
use hyper::{service::Service, Body};

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
        todo!()
    }

    fn call(&mut self, req: EndpointRequest) -> Self::Future {
        println!("call is called, req: {:?}", req);
        todo!()
    }
}
