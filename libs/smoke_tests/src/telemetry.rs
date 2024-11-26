#[cfg(test)]
mod smoke_telemetry {
  use conductor_common::http::ConductorHttpRequest;
  use insta::assert_debug_snapshot;
  use lazy_static::lazy_static;
  use reqwest::Response;
  use serde_json::Value;
  use serial_test::serial;
  use std::env::var;
  use std::time::Duration;
  use tokio::time::sleep;

  lazy_static! {
    static ref CONDUCTOR_URL: String = var("CONDUCTOR_URL").expect("CONDUCTOR_URL env var not set");
  }

  static JAEGER_API: &str = "localhost:16686";
  static ZIPKIN_API: &str = "localhost:9411";

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

  #[derive(Clone, Debug, serde::Deserialize)]
  struct JaegerTracesResponse {
    pub data: Vec<JaegerTrace>,
  }

  #[derive(Clone, Debug, serde::Deserialize)]
  struct JaegerTrace {
    pub spans: Vec<JaegerSpan>,
  }

  #[derive(Clone, Debug, serde::Deserialize)]
  struct JaegerSpan {
    #[serde(rename = "operationName")]
    pub operation_name: String,
  }

  async fn fetch_jaeger_traces(service: &str) -> Vec<JaegerSpan> {
    let url = format!("http://{JAEGER_API}/api/traces?service={service}");

    let response = reqwest::Client::new()
      .get(url)
      .send()
      .await
      .expect("failed to fetch jaeger traces")
      .json::<JaegerTracesResponse>()
      .await
      .expect("failed to get jaeger response");

    assert_eq!(response.data.len(), 1);

    response.data[0].spans.clone()
  }

  #[derive(Clone, Debug, serde::Deserialize)]
  struct ZipkinSpan {
    name: String,
  }

  async fn fetch_zipkin_traces(service: &str) -> Vec<ZipkinSpan> {
    let url = format!("http://{ZIPKIN_API}/api/v2/traces?serviceName={service}");

    let response = reqwest::Client::new()
      .get(url)
      .send()
      .await
      .expect("failed to fetch zipkin traces")
      .json::<Vec<Vec<ZipkinSpan>>>()
      .await
      .expect("failed to get zipkin response");

    assert_eq!(response.len(), 1);

    response[0].clone()
  }

  #[tokio::test]
  #[serial]
  #[cfg(feature = "binary")]
  async fn telemetry_otlp_grpc() {
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/telemetry-jaeger-otlp-grpc", CONDUCTOR_URL.as_str())
      .parse()
      .unwrap();
    let gql_response: Response = make_graphql_request(req).await;
    assert_eq!(gql_response.status(), 200);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);

    sleep(Duration::from_secs(8)).await; // Jaeger needs some processing time...
    let traces = fetch_jaeger_traces("conductor-otlp-test-grpc").await;

    assert!(traces
      .iter()
      .find(|v| v.operation_name == "transform_request")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "transform_response")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "query")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "execute")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "POST /")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "upstream_call")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "graphql_parse")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "HTTP POST /telemetry-jaeger-otlp-grpc")
      .is_some());
  }

  #[tokio::test]
  #[serial]
  async fn telemetry_otlp_http() {
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/telemetry-jaeger-otlp-http", CONDUCTOR_URL.as_str())
      .parse()
      .unwrap();
    let gql_response: Response = make_graphql_request(req).await;
    assert_eq!(gql_response.status(), 200);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);

    sleep(Duration::from_secs(8)).await; // Jaeger needs some processing time...
    let traces = fetch_jaeger_traces("conductor-otlp-test-http").await;

    assert!(traces
      .iter()
      .find(|v| v.operation_name == "transform_request")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "transform_response")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "query")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "execute")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "POST /")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "upstream_call")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "graphql_parse")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.operation_name == "HTTP POST /telemetry-jaeger-otlp-http")
      .is_some());
  }

  #[tokio::test]
  #[serial]
  async fn telemetry_zipkin() {
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/telemetry-zipkin", CONDUCTOR_URL.as_str())
      .parse()
      .unwrap();
    let gql_response: Response = make_graphql_request(req).await;
    assert_eq!(gql_response.status(), 200);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);

    sleep(Duration::from_secs(7)).await; // Zipkin needs some processing time...
    let traces = fetch_zipkin_traces("conductor-zipkin").await;

    assert!(traces
      .iter()
      .find(|v| v.name == "transform_request")
      .is_some());
    assert!(traces
      .iter()
      .find(|v| v.name == "transform_response")
      .is_some());
    assert!(traces.iter().find(|v| v.name == "query").is_some());
    assert!(traces.iter().find(|v| v.name == "execute").is_some());
    assert!(traces.iter().find(|v| v.name == "post /").is_some());
    assert!(traces.iter().find(|v| v.name == "upstream_call").is_some());
    assert!(traces.iter().find(|v| v.name == "graphql_parse").is_some());
    assert!(traces
      .iter()
      .find(|v| v.name == "http post /telemetry-zipkin")
      .is_some());
  }
}
