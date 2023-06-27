use std::pin::Pin;

use async_graphql::Variables;
use hyper::{service::Service, Body};
use serde_json::json;

use crate::config::GraphQLSourceConfig;

#[derive(Debug)]
pub struct SourceRequest {
    pub query: String,
    pub variables: Variables,
    pub operation_name: Option<String>,
}

pub type SourceResponse = hyper::Response<hyper::Body>;
pub type SourceFuture = Pin<
    Box<
        (dyn std::future::Future<Output = Result<SourceResponse, SourceError>>
             + std::marker::Send
             + 'static),
    >,
>;

pub trait SourceService:
    Service<SourceRequest, Response = SourceResponse, Error = SourceError, Future = SourceFuture>
{
    fn create(config: GraphQLSourceConfig) -> Self
    where
        Self: Sized;
}

#[derive(Debug)]
pub enum SourceError {
    UnexpectedHTTPStatusError(hyper::StatusCode),
    NetworkError(hyper::Error),
    InvalidPlannedRequest(hyper::http::Error),
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl SourceRequest {
    pub fn into_hyper_request(
        self,
        endpoint: &String,
    ) -> Result<hyper::Request<Body>, hyper::http::Error> {
        hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(endpoint)
            .header("content-type", "application/json")
            // DOTAN: Should we avoid building a JSON and then stringify it here?
            .body(Body::from(
                json!({
                        "query": self.query,
                        "variables": self.variables,
                        "operationName": self.operation_name,
                })
                .to_string(),
            ))
    }
}
