use std::sync::Arc;

use conductor_common::{
  graphql::GraphQLRequest,
  http::{ConductorHttpRequest, ConductorHttpResponse, HttpHeadersMap, Method, CONTENT_TYPE},
  plugin::Plugin,
};
use conductor_config::GraphQLSourceConfig;
use conductor_engine::{gateway::ConductorGateway, source::graphql_source::GraphQLSourceRuntime};
use httpmock::{prelude::*, Then, When};
use serde_json::json;

#[derive(Default)]
pub struct TestSuite {
  pub plugins: Vec<Box<dyn Plugin>>,
  pub mock_server: Option<MockServer>,
}

impl TestSuite {
  pub async fn run_with_mock(
    self,
    request: ConductorHttpRequest,
    mock_fn: impl FnOnce(When, Then),
  ) -> ConductorHttpResponse {
    let mock_server = self.mock_server.unwrap_or_else(MockServer::start);
    let mock = mock_server.mock(mock_fn);

    let source = GraphQLSourceRuntime::new(GraphQLSourceConfig {
      endpoint: mock_server.url("/graphql"),
    });

    let response = ConductorGateway::execute_test(Arc::new(source), self.plugins, request).await;

    mock.assert();
    response
  }

  pub async fn run_http_request(self, request: ConductorHttpRequest) -> ConductorHttpResponse {
    let mock_server = self.mock_server.unwrap_or_else(MockServer::start);

    mock_server.mock(|when, then| {
      when.method(POST).path("/graphql");
      then
        .status(200)
        .header("content-type", "application/json")
        .body(
          json!({
              "data": {
                  "__typename": "Query"
              },
              "errors": null
          })
          .to_string(),
        );
    });

    let source = GraphQLSourceRuntime::new(GraphQLSourceConfig {
      endpoint: mock_server.url("/graphql"),
    });

    ConductorGateway::execute_test(Arc::new(source), self.plugins, request).await
  }

  pub async fn run_graphql_request(self, request: GraphQLRequest) -> ConductorHttpResponse {
    let mut headers = HttpHeadersMap::new();
    headers.append(CONTENT_TYPE, "application/json".parse().unwrap());
    let request = ConductorHttpRequest {
      method: Method::POST,
      query_string: "".to_string(),
      uri: "/graphql".to_string(),
      body: request.operation.unwrap().into(),
      headers,
    };

    self.run_http_request(request).await
  }
}
