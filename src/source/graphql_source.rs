use std::time::Duration;

use crate::config::GraphQLSourceConfig;
use crate::source::base_source::{SourceError, SourceFuture, SourceRequest, SourceResponse};

use axum::Error;
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;

use super::base_source::SourceService;

#[derive(Debug, Clone)]
pub struct GraphQLSourceService {
    pub fetcher: Client<HttpsConnector<HttpConnector>>,
    pub config: GraphQLSourceConfig,
}

impl GraphQLSourceService {
    pub fn create(config: GraphQLSourceConfig) -> Self {
        Self::from_config(config)
    }

    pub fn from_config(config: GraphQLSourceConfig) -> Self {
        // HttpsConnector(HttpConnector) recommended by Hyper docs: https://hyper.rs/guides/0.14/client/configuration/
        let mut http_connector = HttpConnector::new();
        // DOTAN: Do we need anything socket-related here?
        // see https://stackoverflow.com/questions/3192940/best-socket-options-for-client-and-sever-that-continuously-transfer-data
        http_connector.enforce_http(false);
        // DOTAN: Do we need to set a timeout here? feels like for CONNECT phase is might be too much?
        http_connector.set_connect_timeout(Some(Duration::from_secs(10)));
        // DOTAN: this probably needs to be configurable by the user, per source?
        http_connector.set_keepalive(Some(Duration::from_secs(120)));

        // DOTAN: What about HTTP2?
        // DOTAN: What about proxying?

        let mut https_connector = HttpsConnector::new_with_connector(http_connector);
        https_connector.https_only(false);

        Self {
            fetcher: Client::builder().build(https_connector),
            config,
        }
    }
}

impl SourceService for GraphQLSourceService {
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Error>> {
        // DOTAN: Do we want to implement something else here? Does the service considered "ready" only if the
        // endpoint is reachable and we have instrospection available?

        std::task::Poll::Ready(Ok(()))
    }

    fn call(&self, req: SourceRequest) -> SourceFuture {
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
