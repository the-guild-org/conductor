use std::pin::Pin;

use axum::response::Result;
use axum::Error;
use core::fmt::Debug;
use hyper::Body;
use std::task::Context;

use crate::graphql_utils::GraphQLRequest;

pub type SourceResponse = hyper::Response<hyper::Body>;
pub type SourceFuture<'a> = Pin<
    Box<
        (dyn std::future::Future<Output = Result<SourceResponse, SourceError>>
             + std::marker::Send
             + 'a),
    >,
>;

pub trait SourceService: Send + Sync + Debug + 'static {
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> std::task::Poll<Result<(), Error>>;
    fn call(&self, source_req: GraphQLRequest) -> SourceFuture;
}

#[derive(Debug)]
pub enum SourceError {
    UnexpectedHTTPStatusError(hyper::StatusCode),
    NetworkError(hyper::Error),
    InvalidPlannedRequest(hyper::http::Error),
}

impl GraphQLRequest {
    pub async fn into_hyper_request(
        &self,
        endpoint: &String,
    ) -> Result<hyper::Request<Body>, hyper::http::Error> {
        hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(endpoint)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(self).unwrap()))
    }
}

#[cfg(test)]
#[derive(Debug)]
pub(crate) struct MockedService;

#[cfg(test)]
impl MockedService {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
impl SourceService for MockedService {
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&self, mut _source_req: GraphQLRequest) -> SourceFuture {
        use serde_json::json;

        let body = Body::from(
            json!({
                "data": {
                    "hello": "world"
                }
            })
            .to_string(),
        );

        let res = hyper::Response::builder()
            .status(hyper::StatusCode::OK)
            .body(body)
            .unwrap();

        Box::pin(async move { Ok(SourceResponse::new(res.into_body())) })
    }
}
