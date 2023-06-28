use std::time::Duration;

use crate::config::GraphQLSourceConfig;
use crate::source::source::{SourceError, SourceFuture, SourceRequest, SourceResponse};
use axum::Error;
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;

#[derive(Debug, Clone)]
pub struct GraphQLSourceService {
    pub fetcher: Client<HttpsConnector<HttpConnector>>,
    pub config: GraphQLSourceConfig,
}

impl GraphQLSourceService {
    pub fn from_config(config: GraphQLSourceConfig) -> Self {
        let mut http_connector = HttpConnector::new();
        http_connector.enforce_http(false);
        http_connector.set_connect_timeout(Some(Duration::from_secs(10)));
        http_connector.set_keepalive(Some(Duration::from_secs(120)));

        let mut https_connector = HttpsConnector::new_with_connector(http_connector);
        https_connector.https_only(false);

        Self {
            fetcher: Client::builder().build(https_connector),
            config,
        }
    }

    pub fn create(config: GraphQLSourceConfig) -> Self {
        Self::from_config(config)
    }

    pub fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    pub fn call(&mut self, req: SourceRequest) -> SourceFuture {
        let fetcher = self.fetcher.clone();
        let endpoint = String::from(self.config.endpoint.clone());

        Box::pin(async move {
            let req = req
                .into_hyper_request(&endpoint)
                .await
                .map_err(|e| SourceError::InvalidPlannedRequest(e))?;

            let result = fetcher.request(req).await;

            match result {
                Ok(res) => match res.status() {
                    hyper::StatusCode::OK => Ok(SourceResponse::new(res.into_body())),
                    code => Err(SourceError::UnexpectedHTTPStatusError(code)),
                },
                Err(e) => Err(SourceError::NetworkError(e)),
            }
        })
    }
}
