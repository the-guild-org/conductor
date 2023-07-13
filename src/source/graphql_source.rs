use std::sync::Arc;
use std::time::Duration;

use crate::config::GraphQLSourceConfig;
use crate::plugins::plugin_manager::PluginManager;
use crate::source::base_source::{SourceError, SourceFuture, SourceRequest, SourceResponse};

use axum::Error;
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;

use super::base_source::SourceService;

#[derive(Debug, Clone)]
pub struct GraphQLSourceService {
    pub fetcher: Client<HttpsConnector<HttpConnector>>,
    pub config: GraphQLSourceConfig,
    pub plugin_manager: Arc<PluginManager>,
}

impl GraphQLSourceService {
    pub fn from_config(config: GraphQLSourceConfig, plugin_manager: Arc<PluginManager>) -> Self {
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
            plugin_manager,
        }
    }
}

impl SourceService for GraphQLSourceService {
    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&self, source_req: SourceRequest) -> SourceFuture {
        let fetcher = self.fetcher.clone();
        let endpoint = self.config.endpoint.clone();
        let source_req = self.plugin_manager.on_upstream_graphql_request(source_req);

        Box::pin(async move {
            let req = source_req
                .into_hyper_request(&endpoint)
                .await
                .map_err(SourceError::InvalidPlannedRequest)?;

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
