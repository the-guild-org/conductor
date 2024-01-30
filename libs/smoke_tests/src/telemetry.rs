#[cfg(test)]
#[cfg(not(feature = "wasm"))] // TODO: Remove this when Telemtry will be fully functional on WASM runtime
mod smoke_telemetry {
  use conductor_common::http::ConductorHttpRequest;
  use insta::assert_debug_snapshot;
  use lazy_static::lazy_static;
  use reqwest::Response;
  use serde_json::Value;
  use std::env::var;
  use std::time::{Duration, SystemTime, UNIX_EPOCH};
  use tokio::time::sleep;

  lazy_static! {
    static ref CONDUCTOR_URL: String = var("CONDUCTOR_URL").expect("CONDUCTOR_URL env var not set");
  }

  static JAEGER_API: &str = "localhost:16686";

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

  async fn fetch_traces(service: &str, start: u128, end: u128) -> Vec<JaegerSpan> {
    let url = format!("http://{JAEGER_API}/api/traces?end={end}&limit=20&lookback=1h&maxDuration&minDuration&service={service}&start={start}");

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

  #[tokio::test]
  async fn telemetry_jaeger() {
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/telemetry-jaeger-udp", CONDUCTOR_URL.as_str())
      .parse()
      .unwrap();
    let start_timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_micros();
    let gql_response: Response = make_graphql_request(req).await;
    let end_timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_micros();
    assert_eq!(gql_response.status(), 200);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);

    sleep(Duration::from_secs(5)).await; // Jaeger needs some processing time...
    let traces = fetch_traces("conductor-jaeger-test", start_timestamp, end_timestamp).await;

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
      .find(|v| v.operation_name == "HTTP POST /telemetry-jaeger-udp")
      .is_some());
  }

  #[tokio::test]
  async fn telemetry_otlp() {
    let mut req = ConductorHttpRequest::default();
    req.method = reqwest::Method::POST;
    req.uri = format!("{}/telemetry-jaeger-otlp", CONDUCTOR_URL.as_str())
      .parse()
      .unwrap();
    let start_timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_micros();
    let gql_response: Response = make_graphql_request(req).await;
    let end_timestamp = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_micros();
    assert_eq!(gql_response.status(), 200);
    let json_body = gql_response.json::<Value>().await.unwrap();
    assert_debug_snapshot!(json_body);

    sleep(Duration::from_secs(5)).await; // Jaeger needs some processing time...
    let traces = fetch_traces("conductor-otlp-test", start_timestamp, end_timestamp).await;

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
      .find(|v| v.operation_name == "HTTP POST /telemetry-jaeger-otlp")
      .is_some());
  }
}
