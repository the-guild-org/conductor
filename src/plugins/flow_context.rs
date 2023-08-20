use std::io::ErrorKind;

use async_graphql::{futures_util::TryStreamExt, http::MultipartOptions};
use async_graphql::{http::receive_batch_body, Request as GraphQLRequest};
use axum::extract::BodyStream;
use hyper::{Body, HeaderMap};
use tokio_util::compat::TokioAsyncReadCompatExt;

#[derive(Debug)]
pub struct FlowContext {
    pub downstream_graphql_request: Option<GraphQLRequest>,
    pub downstream_headers: HeaderMap,
    pub short_circuit_response: Option<hyper::Response<Body>>,
}

impl FlowContext {
    #[tracing::instrument(level = "trace")]
    pub async fn extract_graphql_request_from_http_request(
        &mut self,
        body_stream: &mut BodyStream,
    ) {
        let content_type = self
            .downstream_headers
            .get(axum::http::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(ToString::to_string);

        let body = body_stream.map_err(|_| {
            std::io::Error::new(
                ErrorKind::Other,
                "body has been taken by another extractor".to_string(),
            )
        });

        let body_reader = tokio_util::io::StreamReader::new(body).compat();
        let graphql_request =
            receive_batch_body(content_type, body_reader, MultipartOptions::default())
                .await
                .unwrap()
                .into_single()
                .unwrap();

        self.downstream_graphql_request = Some(graphql_request);
    }
}
