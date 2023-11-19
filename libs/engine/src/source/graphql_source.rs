use std::{future::Future, pin::Pin};

use conductor_common::{graphql::GraphQLResponse, http::Bytes};
use conductor_config::GraphQLSourceConfig;
use reqwest::{Client, Method, StatusCode};

use crate::{
    gateway::ConductorGatewayRouteData, request_execution_context::RequestExecutionContext,
};

use super::runtime::{SourceError, SourceRuntime};

#[derive(Debug)]
pub struct GraphQLSourceRuntime {
    pub fetcher: Client,
    pub config: GraphQLSourceConfig,
}

impl GraphQLSourceRuntime {
    pub fn new(config: GraphQLSourceConfig) -> Self {
        let fetcher = wasm_polyfills::create_http_client().build().unwrap();

        Self { fetcher, config }
    }
}

impl SourceRuntime for GraphQLSourceRuntime {
    #[tracing::instrument(
        skip(self, route_data, request_context),
        name = "GraphQLSourceRuntime::execute"
    )]
    fn execute<'a>(
        &'a self,
        route_data: &'a ConductorGatewayRouteData,
        request_context: &'a mut RequestExecutionContext<'_>,
    ) -> Pin<Box<(dyn Future<Output = Result<GraphQLResponse, SourceError>> + Send + 'a)>> {
        Box::pin(wasm_polyfills::call_async(async move {
            let fetcher = &self.fetcher;
            let endpoint = &self.config.endpoint;
            let source_req = &mut request_context
                .downstream_graphql_request
                .as_mut()
                .unwrap()
                .request;
            route_data
                .plugin_manager
                .on_upstream_graphql_request(source_req)
                .await;

            let body_bytes: Bytes = source_req.into();
            let upstream_response = fetcher
                .request(Method::POST, endpoint)
                .body(body_bytes)
                .send()
                .await;

            match upstream_response {
                Ok(res) => match res.status() {
                    StatusCode::OK => {
                        let body = res.bytes().await.unwrap();
                        let mut response =
                            serde_json::from_slice::<GraphQLResponse>(&body).unwrap();

                        route_data
                            .plugin_manager
                            .on_upstream_graphql_response(&mut response)
                            .await;

                        Ok(response)
                    }
                    code => Err(SourceError::UnexpectedHTTPStatusError(code)),
                },
                Err(e) => Err(SourceError::NetworkError(e)),
            }
        }))
    }
}
