use std::pin::Pin;

use async_graphql::Variables;
use hyper::Body;

use axum::response::Result;
use axum::Error;
use serde::Serialize;
use std::task::Context;

#[derive(Serialize, Debug)]
pub struct SourceRequest<'a> {
    query: &'a str,
    variables: Option<&'a Variables>,
    #[serde(rename = "operationName")]
    operation_name: Option<&'a str>,
}

pub type SourceResponse = hyper::Response<hyper::Body>;
pub type SourceFuture<'a> = Pin<
    Box<
        (dyn std::future::Future<Output = Result<SourceResponse, SourceError>>
             + std::marker::Send
             + 'a),
    >,
>;

pub trait SourceService: Send + Sync + 'static {
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> std::task::Poll<Result<(), Error>>;
    fn call<'a>(&'a self, source_req: SourceRequest<'a>) -> SourceFuture<'a>;
}

#[derive(Debug)]
pub enum SourceError {
    UnexpectedHTTPStatusError(hyper::StatusCode),
    NetworkError(hyper::Error),
    InvalidPlannedRequest(hyper::http::Error),
}

impl<'a> SourceRequest<'a> {
    pub fn from_parts(
        operation_name: Option<&'a str>,
        query: &'a str,
        variables: Option<&'a Variables>,
    ) -> Self {
        Self {
            operation_name,
            query,
            variables,
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
            .body(Body::from(serde_json::to_string(&self).unwrap()))
    }
}
