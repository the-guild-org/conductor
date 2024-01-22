use std::time::Duration;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
