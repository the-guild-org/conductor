#[cfg(test)]
mod schema_awareness_smoke {
  use conductor_common::http::ConductorHttpRequest;
  use insta::assert_debug_snapshot;
  use lazy_static::lazy_static;
  use reqwest::Response;
  use serde_json::Value;
  use std::env::var;

  lazy_static! {
    static ref CONDUCTOR_URL: String = var("CONDUCTOR_URL").expect("CONDUCTOR_URL env var not set");
  }

  async fn make_graphql_request(req: ConductorHttpRequest) -> Response {
    let req_builder = reqwest::Client::new()
      .request(req.method, req.uri)
      .headers(req.headers)
      .body(req.body);

    req_builder
      .send()
      .await
      .expect("failed to run http req to conductor")
  }

  #[tokio::test]
  async fn schema_awareness_introspection_ok() {
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/graphql_schema_awareness", CONDUCTOR_URL.as_str())
      .parse()
      .unwrap();
    let gql_response: Response = make_graphql_request(req).await;

    assert_eq!(gql_response.status(), 200);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);
  }

  #[tokio::test]
  async fn schema_awareness_introspection_failed_validation() {
    let mut req = ConductorHttpRequest::default();
    req.body = serde_json::json!({
        "query": "query { __typename invalidField }",
    })
    .to_string()
    .into();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/graphql_schema_awareness", CONDUCTOR_URL.as_str())
      .parse()
      .unwrap();
    let gql_response: Response = make_graphql_request(req).await;

    assert_eq!(gql_response.status(), 200);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);
  }
}
