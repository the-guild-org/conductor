use std::pin::Pin;

use async_graphql::Variables;
use hyper::Body;
use serde_json::{json, Value};

use axum::response::Result;
use axum::Error;
use std::task::Context;

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

pub trait SourceService: Send + Sync + 'static {
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> std::task::Poll<Result<(), Error>>;
    fn call(&self, req: SourceRequest) -> SourceFuture;
}

#[derive(Debug)]
pub enum SourceError {
    UnexpectedHTTPStatusError(hyper::StatusCode),
    NetworkError(hyper::Error),
    InvalidPlannedRequest(hyper::http::Error),
}

impl SourceRequest {
    pub async fn new(body: String) -> Self {
        let req_body: Value = serde_json::from_str(&body)
            .expect("coudln't parse request body, it is maybe corrupted");

        let extract_field_data = |key: &str| {
            req_body
                .get(key)
                .and_then(|v| v.as_str().map(|s| s.to_string()))
        };

        // Yassin: We do need to validate those exist, and if not return an error
        Self {
            operation_name: extract_field_data("operation_name"),
            query: extract_field_data("query").unwrap(),
            variables: match req_body.get("variables") {
                Some(variables) => Some(serde_json::from_value(variables.clone()).unwrap()),
                None => None,
            },
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
