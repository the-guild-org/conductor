use crate::{
    config::{EndpointDefinition, GraphQLSourceConfig},
    source::{
        graphql_source::GraphQLSourceService,
        source::{SourceError, SourceRequest},
    },
};
use hyper::{service::Service, Body};
use std::pin::Pin;

pub type EndpointRequest = hyper::Request<Body>;
pub type EndpointResponse = hyper::Response<Body>;
pub type EndpointError = SourceError;

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
    type Future = Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>,
    >;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: EndpointRequest) -> Self::Future {
        let cloned_config = self.config.clone();
        Box::pin(async move {
            let source_request = SourceRequest::new(req).await;
            let future = GraphQLSourceService::from_config(GraphQLSourceConfig {
                endpoint: cloned_config.from,
            })
            .call(source_request);

            future.await
        })
    }
}
