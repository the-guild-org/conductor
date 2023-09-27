use axum::{body::BoxBody, response::IntoResponse};
use http::{Request, Response};

use crate::{endpoint::endpoint_runtime::EndpointRuntime, graphql_utils::ParsedGraphQLRequest};
use hyper::Body;

#[derive(Debug)]
pub struct FlowContext<'a> {
    pub endpoint: Option<&'a EndpointRuntime>,
    pub downstream_graphql_request: Option<ParsedGraphQLRequest>,
    pub downstream_http_request: &'a mut Request<Body>,
    pub short_circuit_response: Option<Response<BoxBody>>,
}

impl<'a> FlowContext<'a> {
    pub fn new(endpoint: &'a EndpointRuntime, request: &'a mut Request<Body>) -> Self {
        FlowContext {
            downstream_graphql_request: None,
            downstream_http_request: request,
            short_circuit_response: None,
            endpoint: Some(endpoint),
        }
    }

    #[cfg(test)]
    pub fn empty_from_request(request: &'a mut Request<Body>) -> Self {
        FlowContext {
            downstream_graphql_request: None,
            downstream_http_request: request,
            short_circuit_response: None,
            endpoint: None,
        }
    }

    pub fn short_circuit(&mut self, response: impl IntoResponse) {
        self.short_circuit_response = Some(response.into_response());
    }

    pub fn is_short_circuit(&self) -> bool {
        self.short_circuit_response.is_some()
    }

    pub fn has_failed_extraction(&self) -> bool {
        self.downstream_graphql_request.is_none()
    }
}
