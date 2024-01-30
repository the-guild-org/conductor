#[cfg(test)]
mod smoke_jwt {
  use conductor_common::http::ConductorHttpRequest;
  use insta::assert_debug_snapshot;
  use lazy_static::lazy_static;
  use reqwest::header::CONTENT_TYPE;
  use reqwest::Response;
  use serde_json::Value;
  use std::collections::HashMap;
  use std::env::var;

  lazy_static! {
    static ref CONDUCTOR_URL: String = var("CONDUCTOR_URL").expect("CONDUCTOR_URL env var not set");
  }

  static KEYCLOAK_URL: &str = "http://localhost:4001";

  #[derive(serde::Deserialize)]
  struct JwtMock {
    access_token: String,
  }

  async fn create_token() -> String {
    let mut form_params = HashMap::<&str, &str>::new();
    form_params.insert("client_id", "conductor");
    form_params.insert("grant_type", "password");
    // Should match /libs/smoke_tests/volumes/keycloak/realm.json client secret
    form_params.insert("client_secret", "dlApaPkSI9xPL4gG3HLnIU8MnB66eJOz");
    form_params.insert("scope", "openid");
    form_params.insert("username", "test");
    form_params.insert("password", "test");

    let response = reqwest::Client::new()
      .post(format!(
        "{KEYCLOAK_URL}/realms/test/protocol/openid-connect/token"
      ))
      .form(&form_params)
      .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
      .send()
      .await
      .expect("failed to create jwt token");

    if !response.status().is_success() {
      panic!(
        "failed to create jwt token, status: {}, body: {:?}",
        response.status(),
        response.text().await
      );
    }

    response.json::<JwtMock>().await.unwrap().access_token
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
  async fn valid_token() {
    let token = create_token().await;
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/jwt", CONDUCTOR_URL.as_str()).parse().unwrap();
    req.headers.append(
      "authorization",
      format!("Bearer {}", token).parse().unwrap(),
    );
    let gql_response = make_graphql_request(req).await;
    assert_eq!(gql_response.status(), 200);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);
  }

  #[tokio::test]
  async fn invalid_token_type() {
    let token = create_token().await;
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/jwt", CONDUCTOR_URL.as_str()).parse().unwrap();
    req.headers.append(
      "authorization",
      format!("TokenType {}", token).parse().unwrap(),
    );
    let gql_response = make_graphql_request(req).await;
    assert_eq!(gql_response.status(), 400);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);
  }

  #[tokio::test]
  async fn token_header_missing() {
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/jwt", CONDUCTOR_URL.as_str()).parse().unwrap();
    req
      .headers
      .append("authorization", format!("Bearer ").parse().unwrap());
    let gql_response = make_graphql_request(req).await;
    assert_eq!(gql_response.status(), 400);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);
  }

  #[tokio::test]
  async fn empty_token() {
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/jwt", CONDUCTOR_URL.as_str()).parse().unwrap();
    let gql_response = make_graphql_request(req).await;
    assert_eq!(gql_response.status(), 400);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);
  }

  #[tokio::test]
  async fn invalid_token() {
    let token = "bad jwt";
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/jwt", CONDUCTOR_URL.as_str()).parse().unwrap();
    req.headers.append(
      "authorization",
      format!("Bearer {}", token).parse().unwrap(),
    );
    let gql_response = make_graphql_request(req).await;
    assert_eq!(gql_response.status(), 400);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);
  }

  #[tokio::test]
  async fn invalid_audience() {
    // Same as valid token, but has audience set to "bad_aud"
    let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImYwN0FUa0tNUUZrWWVkQ2pXaFdCRG1yWG5EaFJGVDV0bmxwOUh5Z0FpMUEifQ.eyJleHAiOjE4MDY0NTUwMTMsImlhdCI6MTcwNjQ1NDk1MywianRpIjoiYTMzYzZjMjktOWMxYS00MTRmLTgxOTUtNDk2YjBhZGNkYzZjIiwiaXNzIjoiaHR0cDovL2xvY2FsaG9zdDo0MDAxL3JlYWxtcy9tYXN0ZXIiLCJhdWQiOiJiYWRfYXVkIiwic3ViIjoiZGRlZGMwNjItOTc0NC00ZGVhLThhOGQtNjk3YWIxMmNkNTBjIiwidHlwIjoiQmVhcmVyIiwiYXpwIjoiY29uZHVjdG9yIiwic2Vzc2lvbl9zdGF0ZSI6ImQ3YzYyYjE2LWI0NmEtNDdjMC1iZDYwLWJkYTk5OWM4MjY2MiIsImFjciI6IjEiLCJhbGxvd2VkLW9yaWdpbnMiOlsiLyoiXSwicmVhbG1fYWNjZXNzIjp7InJvbGVzIjpbImRlZmF1bHQtcm9sZXMtbWFzdGVyIiwib2ZmbGluZV9hY2Nlc3MiLCJ1bWFfYXV0aG9yaXphdGlvbiJdfSwicmVzb3VyY2VfYWNjZXNzIjp7ImFjY291bnQiOnsicm9sZXMiOlsibWFuYWdlLWFjY291bnQiLCJtYW5hZ2UtYWNjb3VudC1saW5rcyIsInZpZXctcHJvZmlsZSJdfX0sInNjb3BlIjoib3BlbmlkIHByb2ZpbGUgZW1haWwiLCJzaWQiOiJkN2M2MmIxNi1iNDZhLTQ3YzAtYmQ2MC1iZGE5OTljODI2NjIiLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwibmFtZSI6InRlc3QgdGVzdCIsInByZWZlcnJlZF91c2VybmFtZSI6InRlc3QiLCJnaXZlbl9uYW1lIjoidGVzdCIsImZhbWlseV9uYW1lIjoidGVzdCIsImVtYWlsIjoidGVzdEB0ZXN0LmNvbSJ9.g3BXU0esmeB4g47UdxtXdNFdmPuv2YR-GxJY_uiyS8WrEB727JQc5tHpHpAhWZHY36lkJ8e7Mly13eeR1AVoEqfmr3RHWlCppNQH4q3iE5wS3FFSlv44uPpzTawvmdITMTXZGY86-RTUp2TlJ92IBztu-BeCj5XEUi60E3TJ6TBj6kS-HbJeDCv4neRcg_-dJ2GPPsO1ob0mdk_I5MHRwDMBog4XQpgiqiNC6rUiXnyqIVCjIgRBogswv7pbS44P4m5-N2_DyF15FuKsPWLApByI32ona5SZNemTrJHgA5jatBsBz2weFZurJSniyn6YKJWHcsV7D6cOComxVjtmJA";
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/jwt", CONDUCTOR_URL.as_str()).parse().unwrap();
    req.headers.append(
      "authorization",
      format!("Bearer {}", token).parse().unwrap(),
    );
    let gql_response = make_graphql_request(req).await;
    assert_eq!(gql_response.status(), 400);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);
  }

  #[tokio::test]
  async fn invalid_issuer() {
    // Same as valid token, but has audience set to "bad_aud"
    let token = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6ImYwN0FUa0tNUUZrWWVkQ2pXaFdCRG1yWG5EaFJGVDV0bmxwOUh5Z0FpMUEifQ.eyJleHAiOjE4MDY0NTUwMTMsImlhdCI6MTcwNjQ1NDk1MywianRpIjoiYTMzYzZjMjktOWMxYS00MTRmLTgxOTUtNDk2YjBhZGNkYzZjIiwiaXNzIjoiaHR0cDovL290aGVyLmNvbS9yZWFsbXMvbWFzdGVyIiwiYXVkIjoiYWNjb3VudCIsInN1YiI6ImRkZWRjMDYyLTk3NDQtNGRlYS04YThkLTY5N2FiMTJjZDUwYyIsInR5cCI6IkJlYXJlciIsImF6cCI6ImNvbmR1Y3RvciIsInNlc3Npb25fc3RhdGUiOiJkN2M2MmIxNi1iNDZhLTQ3YzAtYmQ2MC1iZGE5OTljODI2NjIiLCJhY3IiOiIxIiwiYWxsb3dlZC1vcmlnaW5zIjpbIi8qIl0sInJlYWxtX2FjY2VzcyI6eyJyb2xlcyI6WyJkZWZhdWx0LXJvbGVzLW1hc3RlciIsIm9mZmxpbmVfYWNjZXNzIiwidW1hX2F1dGhvcml6YXRpb24iXX0sInJlc291cmNlX2FjY2VzcyI6eyJhY2NvdW50Ijp7InJvbGVzIjpbIm1hbmFnZS1hY2NvdW50IiwibWFuYWdlLWFjY291bnQtbGlua3MiLCJ2aWV3LXByb2ZpbGUiXX19LCJzY29wZSI6Im9wZW5pZCBwcm9maWxlIGVtYWlsIiwic2lkIjoiZDdjNjJiMTYtYjQ2YS00N2MwLWJkNjAtYmRhOTk5YzgyNjYyIiwiZW1haWxfdmVyaWZpZWQiOnRydWUsIm5hbWUiOiJ0ZXN0IHRlc3QiLCJwcmVmZXJyZWRfdXNlcm5hbWUiOiJ0ZXN0IiwiZ2l2ZW5fbmFtZSI6InRlc3QiLCJmYW1pbHlfbmFtZSI6InRlc3QiLCJlbWFpbCI6InRlc3RAdGVzdC5jb20ifQ.Pu6z9WW-MoE2C_xXNotwIe6ggD8FTkHmEBt51s9IjCLGImtvP8mfEhUUddHFbfWc5HBE8uD5LXYv8k6-KIukbnnADd5gLuumxbeF0FQVLvZbaFSH_U4gMp3OJhmGf7S7XKgUmSIO8v9Ax2AHBqmzgfcqmoPcRGCh_aRw8ue9Xye0tJ6f3VQYGZrc1T6rSuf2T1STxh-m_QAPrT0C2NcgQKHOQG4ZY60U9xjNpF1Kq5139v7HKs89qwHv99vj6eBhRGr6dL6DUrmkE93qfNFm4IykkLk8ELoRm3mGAmykPygEYhRb-gxRFG2--2-zqRGN51qmX9TGY6ZQWHvD-SQgNg";
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/jwt", CONDUCTOR_URL.as_str()).parse().unwrap();
    req.headers.append(
      "authorization",
      format!("Bearer {}", token).parse().unwrap(),
    );
    let gql_response = make_graphql_request(req).await;
    assert_eq!(gql_response.status(), 400);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);
  }

  #[tokio::test]
  async fn nonsecure_missing_token() {
    // This one checks against jwt-nonsecure and it allows access without token
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/jwt-nonsecure", CONDUCTOR_URL.as_str())
      .parse()
      .unwrap();
    let gql_response = make_graphql_request(req).await;
    assert_eq!(gql_response.status(), 200);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);
  }
}
