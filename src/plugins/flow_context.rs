use axum::{body::BoxBody, response::IntoResponse};
use http::{Request, Response};
use serde::de::DeserializeOwned;
use serde_json::from_slice;

use crate::{
    endpoint::endpoint_runtime::EndpointRuntime, graphql_utils::ParsedGraphQLRequest,
    http_utils::ExtractGraphQLOperationError,
};
use hyper::{body::to_bytes, Body};

#[derive(Debug)]
pub struct FlowContext<'a> {
    pub endpoint: Option<&'a EndpointRuntime>,
    pub downstream_graphql_request: Option<ParsedGraphQLRequest>,
    pub downstream_http_request: &'a mut Request<Body>,
    pub short_circuit_response: Option<Response<BoxBody>>,
    pub downstream_request_body_bytes: Option<Result<tokio_util::bytes::Bytes, hyper::Error>>,
}

impl<'a> FlowContext<'a> {
    pub fn new(endpoint: &'a EndpointRuntime, request: &'a mut Request<Body>) -> Self {
        FlowContext {
            downstream_graphql_request: None,
            downstream_http_request: request,
            short_circuit_response: None,
            endpoint: Some(endpoint),
            downstream_request_body_bytes: None,
        }
    }

    pub async fn consume_body(&mut self) -> &Result<tokio_util::bytes::Bytes, hyper::Error> {
        if self.downstream_request_body_bytes.is_none() {
            self.downstream_request_body_bytes =
                Some(to_bytes(self.downstream_http_request.body_mut()).await);
        }

        return self.downstream_request_body_bytes.as_ref().unwrap();
    }

    pub async fn json_body<T>(&mut self) -> Result<T, ExtractGraphQLOperationError>
    where
        T: DeserializeOwned,
    {
        let body_bytes = self.consume_body().await;

        match body_bytes {
            Ok(bytes) => {
                let json = from_slice::<T>(bytes)
                    .map_err(ExtractGraphQLOperationError::InvalidBodyJsonFormat)?;

                Ok(json)
            }
            Err(_e) => Err(ExtractGraphQLOperationError::FailedToReadRequestBody),
        }
    }

    #[cfg(test)]
    pub fn empty_from_request(request: &'a mut Request<Body>) -> Self {
        FlowContext {
            downstream_graphql_request: None,
            downstream_http_request: request,
            short_circuit_response: None,
            endpoint: None,
            downstream_request_body_bytes: None,
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
