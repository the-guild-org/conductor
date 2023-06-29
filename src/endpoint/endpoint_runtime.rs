use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    config::EndpointDefinition,
    source::{
        base_source::{SourceError, SourceRequest, SourceService},
        graphql_source::GraphQLSourceService,
    },
};
use async_graphql::parser::Error;
use hyper::Body;

pub type EndpointResponse = hyper::Response<Body>;

#[derive(Clone)]
pub struct EndpointRuntime {
    pub config: EndpointDefinition,
    // pub sources: Arc<HashMap<String, Arc<Mutex<GraphQLSourceService>>>>,
    pub upstream_service: Arc<Mutex<dyn SourceService + Send>>,
}

impl EndpointRuntime {
    pub fn new(
        config: EndpointDefinition,
        sources: Arc<HashMap<String, GraphQLSourceService>>,
    ) -> Self {
        let upstream_service = match sources.get(&config.from) {
            Some(e) => e.to_owned(),
            None => {
                panic!("source {} not found!", config.from);
            }
        };

        Self {
            config,
            upstream_service: Arc::new(Mutex::new(upstream_service)),
        }
    }

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    pub async fn call(&self, body: String) -> Result<EndpointResponse, SourceError> {
        let source_request = SourceRequest::new(body).await;

        let future = self
            .upstream_service
            .lock()
            .expect("upstream service lock coudln't be acquired")
            .call(source_request);
        return future.await;
    }
}
