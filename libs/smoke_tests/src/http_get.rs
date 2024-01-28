#[cfg(test)]
mod smoke_http_get {
  use insta::assert_debug_snapshot;
  use lazy_static::lazy_static;
  use reqwest::header::{HeaderMap, CONTENT_TYPE};
  use serde_json::Value;
  use std::env::var;

  lazy_static! {
    static ref CONDUCTOR_URL: String = var("CONDUCTOR_URL").expect("CONDUCTOR_URL env var not set");
  }

  #[tokio::test]
  async fn http_get() {
    let mut headers = HeaderMap::default();
    headers.append(
      CONTENT_TYPE,
      "application/x-www-form-urlencoded".parse().unwrap(),
    );

    let req = reqwest::Client::new()
      .request(
        reqwest::Method::GET,
        format!(
          "{}/http-get?query=query%20%7B%20__typename%20%7D",
          CONDUCTOR_URL.as_str()
        ),
      )
      .headers(headers);

    let gql_response = req
      .send()
      .await
      .expect("failed to run http req to conductor");
    assert_eq!(gql_response.status(), 200);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);
  }
}
