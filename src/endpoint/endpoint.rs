use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    config::EndpointDefinition,
    source::{
        graphql_source::GraphQLSourceService,
        source::{SourceError, SourceRequest},
    },
};
use async_graphql::parser::Error;
use hyper::{Body, StatusCode};

pub type EndpointRequest = hyper::Request<Body>;
pub type EndpointResponse = hyper::Response<Body>;
pub type EndpointError = SourceError;

#[derive(Clone, Debug)]
pub struct EndpointRuntime {
    pub config: EndpointDefinition,
    pub sources: Arc<HashMap<String, Arc<Mutex<GraphQLSourceService>>>>,
}

impl EndpointRuntime {
    pub fn new(
        config: EndpointDefinition,
        sources: Arc<HashMap<String, GraphQLSourceService>>,
    ) -> Self {
        Self {
            config,
            sources: Arc::new(
                sources
                    .iter()
                    .map::<(String, Arc<Mutex<GraphQLSourceService>>), _>(|source_config| {
                        (
                            source_config.0.clone(),
                            Arc::new(Mutex::new(source_config.1.clone())),
                        )
                    })
                    .collect(),
            ),
        }
    }

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    pub async fn call(&self, body: String) -> Result<EndpointResponse, SourceError> {
        let source_request: SourceRequest = SourceRequest::new(body).await;

        if let Some(source_service) = self.sources.get(&self.config.from) {
            let mut source_service = source_service.lock().unwrap();
            let future = source_service.call(source_request);
            return future.await;
        }

        Err(SourceError::UnexpectedHTTPStatusError(
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
