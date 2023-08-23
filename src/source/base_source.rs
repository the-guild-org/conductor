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
