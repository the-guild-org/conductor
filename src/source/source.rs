use std::pin::Pin;

use async_graphql::Variables;
use hyper::{service::Service, Body};
use serde_json::{json, Value};

use crate::config::GraphQLSourceConfig;
use axum::http::Request;
use axum::response::Result;

#[derive(Debug)]
pub struct SourceRequest {
    query: String,
    variables: Option<Variables>,
    operation_name: Option<String>,
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

pub async fn parse_body_to_string(req: Request<Body>) -> String {
    let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();

    String::from_utf8(body_bytes.to_vec()).unwrap()
}

impl SourceRequest {
    pub async fn new(body: String) -> Self {
        let req_body: Value = serde_json::from_str(&body).unwrap();

        Self {
            operation_name: req_body
                .get("operation_name")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            query: req_body
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            variables: req_body
                .get("variables")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
        }
    }

    pub async fn into_hyper_request(
        self,
        endpoint: &String,
    ) -> Result<hyper::Request<Body>, hyper::http::Error> {
        hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(endpoint)
            .header("content-type", "application/json")
            // DOTAN: Should we avoid building a JSON and then stringify it here?
            // Yassin: Yes
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
