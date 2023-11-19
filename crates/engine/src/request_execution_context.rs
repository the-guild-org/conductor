use conductor_common::{
    graphql::ParsedGraphQLRequest,
    http::{ConductorHttpRequest, ConductorHttpResponse},
};

use crate::endpoint_runtime::EndpointRuntime;

#[derive(Debug)]
pub struct RequestExecutionContext<'a> {
    pub endpoint: &'a EndpointRuntime,
    pub downstream_http_request: ConductorHttpRequest,
    pub downstream_graphql_request: Option<ParsedGraphQLRequest>,
    pub short_circuit_response: Option<ConductorHttpResponse>,
}

impl<'a> RequestExecutionContext<'a> {
    pub fn new(
        endpoint: &'a EndpointRuntime,
        downstream_http_request: ConductorHttpRequest,
    ) -> Self {
        RequestExecutionContext {
            endpoint,
            downstream_http_request,
            downstream_graphql_request: None,
            short_circuit_response: None,
        }
    }

    pub fn short_circuit(&mut self, response: ConductorHttpResponse) {
        self.short_circuit_response = Some(response);
    }

    pub fn is_short_circuit(&self) -> bool {
        self.short_circuit_response.is_some()
    }

    pub fn has_failed_extraction(&self) -> bool {
        self.downstream_graphql_request.is_none()
    }
}
