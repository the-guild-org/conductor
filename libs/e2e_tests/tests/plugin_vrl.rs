use conductor_common::{
  http::{ConductorHttpRequest, HeaderValue, HttpHeadersMap, Method, StatusCode},
  plugin::CreatablePlugin,
  vrl_utils::VrlConfigReference,
};
use e2e::suite::TestSuite;
use httpmock::prelude::*;
use serde_json::json;
use tokio::test;

#[test]
async fn complete_flow_with_shared_state() {
  let plugin = vrl_plugin::Plugin::create(vrl_plugin::Config {
    on_downstream_http_request: Some(VrlConfigReference::Inline {
      content: String::from(
        r#"
                        authorization_header = %downstream_http_req.headers.authorization
                    "#,
      ),
    }),
    on_downstream_graphql_request: Some(VrlConfigReference::Inline {
      content: String::from(
        r#"
                        test = join!(["incoming query:", authorization_header])
                        log(test, level:"info")
                    "#,
      ),
    }),
    on_upstream_http_request: Some(VrlConfigReference::Inline {
      content: String::from(
        r#"
                        .upstream_http_req.headers."x-authorization" = "test-value"
                    "#,
      ),
    }),
    on_downstream_http_response: Some(VrlConfigReference::Inline {
      content: String::from(
        r#"
                        .downstream_http_res.headers."dotan-test" = "oopsi"

                    "#,
      ),
    }),
  })
  .await
  .unwrap();

  let mut header_map = HttpHeadersMap::default();
  header_map.append("content-type", HeaderValue::from_static("application/json"));
  header_map.append(
    "authorization",
    HeaderValue::from_static("application/json"),
  );
  let request: ConductorHttpRequest = ConductorHttpRequest {
    body: "{\"query\": \"query { __typename }\"}".into(),
    uri: String::from("/graphql"),
    query_string: String::from(""),
    method: Method::POST,
    headers: header_map,
  };

  let http_mock = MockServer::start();

  http_mock.mock(|when, then| {
    when
      .method(POST)
      .path("/graphql")
      .header_exists("x-authorization");
    then
      .status(200)
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

  let test = TestSuite {
    plugins: vec![plugin],
    mock_server: Some(http_mock),
  };

  let response = test.run_http_request(request).await;
  assert_eq!(response.status, StatusCode::OK);
  assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}");
  assert!(response
    .headers
    .get("dotan-test")
    .is_some_and(|v| v == "oopsi"));
}

#[test]
#[should_panic]
async fn test_ignore_invalid_vrl_compile() {
  vrl_plugin::Plugin::create(vrl_plugin::Config {
    on_downstream_http_request: Some(VrlConfigReference::Inline {
      // invalid because "b" doesn't exists
      content: String::from(
        r#"
                        a = b
                    "#,
      ),
    }),
    on_downstream_graphql_request: None,
    on_upstream_http_request: None,
    on_downstream_http_response: None,
  })
  .await
  .unwrap();
}

#[test]
async fn test_waterfall_of_hooks() {
  let mut header_map = HttpHeadersMap::default();
  header_map.append("content-type", HeaderValue::from_static("application/json"));
  let request: ConductorHttpRequest = ConductorHttpRequest {
    body: "{\"query\": \"query { __typename }\"}".into(),
    uri: String::from("/graphql"),
    query_string: String::from(""),
    method: Method::POST,
    headers: header_map,
  };

  let plugin = vrl_plugin::Plugin::create(vrl_plugin::Config {
    on_downstream_http_request: Some(VrlConfigReference::Inline {
      content: String::from(
        r#"
                        var1 = "1"
                    "#,
      ),
    }),
    on_downstream_graphql_request: Some(VrlConfigReference::Inline {
      content: String::from(
        r#"
                        var2 = var1 + "2"
                    "#,
      ),
    }),
    on_upstream_http_request: Some(VrlConfigReference::Inline {
      content: String::from(
        r#"
                        var3 = var1 + var2 + "3"
                    "#,
      ),
    }),
    on_downstream_http_response: Some(VrlConfigReference::Inline {
      content: String::from(
        r#"
                        var4 = var1 + var2 + var3 + "4"
                        assert!(var4 == "11211234", message: "invalid value")
                    "#,
      ),
    }),
  })
  .await
  .unwrap();

  let test = TestSuite {
    plugins: vec![plugin],
    mock_server: None,
  };

  let response = test.run_http_request(request.clone()).await;
  assert_eq!(response.status, StatusCode::OK);
  assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}",);

  let plugin = vrl_plugin::Plugin::create(vrl_plugin::Config {
    on_downstream_http_request: Some(VrlConfigReference::Inline {
      content: String::from(
        r#"
                        var1 = "1"
                    "#,
      ),
    }),
    on_downstream_graphql_request: None,
    on_upstream_http_request: None,
    on_downstream_http_response: Some(VrlConfigReference::Inline {
      content: String::from(
        r#"
                        var2 = var1 + "2"
                        assert!(var2 == "12", message: "invalid value")
                    "#,
      ),
    }),
  })
  .await
  .unwrap();
  let test = TestSuite {
    plugins: vec![plugin],
    mock_server: None,
  };
  let response = test.run_http_request(request).await;
  assert_eq!(response.status, StatusCode::OK);
  assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}",);
}

#[test]
async fn test_vrl_on_downstream_request_input_output() {
  let plugin = vrl_plugin::Plugin::create(vrl_plugin::Config {
        on_downstream_http_request: Some(VrlConfigReference::Inline {
            content: String::from(
                r#"
                        # input
                        assert!(%downstream_http_req.headers.authorization == "Bearer XYZ", message: "invalid value")
                        assert!(%downstream_http_req.headers."content-type" == "application/json", message: "invalid value")
                        assert!(%downstream_http_req.method == "POST", message: "invalid value")
                        assert!(%downstream_http_req.body == "{\"query\": \"query { __typename }\"}", message: "invalid value")
                        assert!(%downstream_http_req.uri == "/graphql", message: "invalid value")
                        assert!(%downstream_http_req.query_string == "test=1", message: "invalid value")

                        # output
                        .graphql.operation = "query override { __typename }"
                        .graphql.operation_name = "override"
                    "#,
            ),
        }),
        on_downstream_graphql_request: None,
        on_upstream_http_request: None,
        on_downstream_http_response: None,
    }).await.unwrap();

  let mut header_map = HttpHeadersMap::default();
  header_map.append("Authorization", HeaderValue::from_static("Bearer XYZ"));
  header_map.append("content-type", HeaderValue::from_static("application/json"));
  let request: ConductorHttpRequest = ConductorHttpRequest {
    body: "{\"query\": \"query { __typename }\"}".into(),
    uri: String::from("/graphql"),
    query_string: String::from("test=1"),
    method: Method::POST,
    headers: header_map,
  };

  let test = TestSuite {
    plugins: vec![plugin],
    mock_server: None,
  };

  let response = test.run_http_request(request).await;
  assert_eq!(response.status, StatusCode::OK);
  // In case of a VRL evaluation error, we should fail to short_circuit,
  // so it's safe to use assertion in VRL and check this condition here
  assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}");
}
