use std::time::Duration;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Deserialize, Serialize, Debug, Clone, Default, JsonSchema)]
// TODO: examples
// TODO: docs
pub struct TelemetryPluginConfig {
  #[serde(default = "default_service_name")]
  pub service_name: String,
  pub targets: Vec<OpenTelemetryTarget>,
}

fn default_service_name() -> String {
  "conductor".to_string()
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum OpenTelemetryTarget {
  #[serde(rename = "stdout")]
  #[schemars(title = "stdout")]
  Stdout,
  #[serde(rename = "otlp")]
  #[schemars(title = "Open Telemetry (OTLP)")]
  Otlp {
    endpoint: String,
    #[serde(default = "default_otlp_protocol")]
    protocol: OtlpProtcol,
    #[serde(
      deserialize_with = "humantime_serde::deserialize",
      serialize_with = "humantime_serde::serialize",
      default = "default_otlp_timeout"
    )]
    timeout: Duration,
    #[serde(default)]
    gzip_compression: bool,
  },
  #[serde(rename = "datadog")]
  #[schemars(title = "Datadog")]
  Datadog {
    #[serde(default = "default_datadog_agent_endpoint")]
    agent_endpoint: SocketAddr,
  },
  #[serde(rename = "jaeger")]
  #[schemars(title = "Jaeger")]
  Jaeger {
    #[serde(default = "default_jaeger_endpoint")]
    endpoint: SocketAddr,
  },
}

fn default_jaeger_endpoint() -> SocketAddr {
  "127.0.0.1:6831".parse().unwrap()
}

fn default_datadog_agent_endpoint() -> SocketAddr {
  "127.0.0.1:8126".parse().unwrap()
}

fn default_otlp_protocol() -> OtlpProtcol {
  OtlpProtcol::Grpc
}

fn default_otlp_timeout() -> Duration {
  Duration::from_secs(10)
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub enum OtlpProtcol {
  #[serde(rename = "grpc")]
  Grpc,
  #[serde(rename = "http")]
  Http,
}

impl From<OtlpProtcol> for opentelemetry_otlp::Protocol {
  fn from(value: OtlpProtcol) -> Self {
    match value {
      OtlpProtcol::Grpc => opentelemetry_otlp::Protocol::Grpc,
      OtlpProtcol::Http => opentelemetry_otlp::Protocol::HttpBinary,
    }
  }
}
