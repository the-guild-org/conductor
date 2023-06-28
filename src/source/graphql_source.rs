use std::time::Duration;

use crate::config::GraphQLSourceConfig;
use crate::source::source::{
    SourceError, SourceFuture, SourceRequest, SourceResponse, SourceService,
};
use hyper::{client::HttpConnector, service::Service, Client};
use hyper_tls::HttpsConnector;

#[derive(Debug)]
pub struct GraphQLSourceService {
    pub fetcher: hyper::Client<HttpsConnector<HttpConnector>>,
    pub config: GraphQLSourceConfig,
}

impl GraphQLSourceService {
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
    fn create(config: GraphQLSourceConfig) -> Self {
        Self::from_config(config)
    }
}

impl Service<SourceRequest> for GraphQLSourceService {
    type Response = SourceResponse;
    type Error = SourceError;
    type Future = SourceFuture;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        // DOTAN: Do we want to implement something else here? Does the service considered "ready" only if the
        // endpoint is reachable and we have instrospection available?
        self.fetcher
            .poll_ready(cx)
            .map_err(|e| SourceError::NetworkError(e))
    }

    fn call(&mut self, req: SourceRequest) -> Self::Future {
        let fetcher = self.fetcher.clone();
        let endpoint = self.config.endpoint.clone();

        return Box::pin(async move {
            let req = req
                .into_hyper_request(&endpoint)
                .await
                .map_err(|e| SourceError::InvalidPlannedRequest(e))?;

            let result = fetcher.request(req).await;

            match result {
                Ok(res) => match res.status() {
                    hyper::StatusCode::OK => return Ok(res),
                    code => return Result::Err(SourceError::UnexpectedHTTPStatusError(code)),
                },
                Err(e) => return Result::Err(SourceError::NetworkError(e)),
            }
        });
    }
}
