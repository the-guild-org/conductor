use std::sync::Arc;

use conductor_common::{
    graphql::GraphQLRequest,
    http::{ConductorHttpRequest, ConductorHttpResponse, HttpHeadersMap, Method, CONTENT_TYPE},
    plugin::Plugin,
};
use conductor_config::GraphQLSourceConfig;
use conductor_engine::{
    endpoint_runtime::EndpointRuntime, gateway::ConductorGateway,
    source::graphql_source::GraphQLSourceRuntime,
};
use httpmock::prelude::*;
use serde_json::json;
pub struct TestSuite {
    pub plugins: Vec<Box<dyn Plugin>>,
    pub mock_server: Option<MockServer>,
}

impl Default for TestSuite {
    fn default() -> Self {
        Self {
            plugins: vec![],
            mock_server: None,
        }
    }
}

impl TestSuite {
    pub async fn run_http_request(self, request: ConductorHttpRequest) -> ConductorHttpResponse {
        let mock_server = self.mock_server.unwrap_or_else(|| {
            let http_mock = MockServer::start();
            http_mock.mock(|when, then| {
                when.method(POST).path("/graphql");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(
                        &json!({
                            "data": {
                                "__typename": "Query"
                            },
                            "errors": null
                        })
                        .to_string(),
                    );
            });

            http_mock
        });
        let source = GraphQLSourceRuntime::new(GraphQLSourceConfig {
            endpoint: mock_server.url("/graphql"),
        });

        ConductorGateway::execute_test(
            EndpointRuntime::dummy(),
            Arc::new(source),
            self.plugins,
            request,
        )
        .await
    }

    pub async fn run_graphql_request(self, request: GraphQLRequest) -> ConductorHttpResponse {
        let mut headers = HttpHeadersMap::new();
        headers.append(CONTENT_TYPE, "application/json".parse().unwrap());
        let request = ConductorHttpRequest {
            method: Method::POST,
            query_string: "".to_string(),
            uri: "/graphql".to_string(),
            body: request.to_string().into(),
            headers,
        };

        self.run_http_request(request).await
    }
}
