use std::time::Duration;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Deserialize, Serialize, Debug, Clone, Default, JsonSchema)]
/// The `telemetry` plugin exports traces information about Conductor to a telemetry backend.
///
/// The telemetry plugin exports traces information about the following aspects of Conductor:
///
/// - GraphQL parser (timing)
///
/// - GraphQL execution (operation type, operation body, operation name, timing, errors)
///
/// - Query planning (timing, operation body, operation name)
///
/// - Incoming HTTP requests (attributes, timing, errors)
///
/// - Outgoing HTTP requests (attributes, timing, errors)
///
/// When used with a telemtry backend, you can expect to see the following information:
///
/// ![img](https://raw.githubusercontent.com/the-guild-org/conductor/master/website/public/assets/telemetry.png)
///
pub struct TelemetryPluginConfig {
  /// Configures the service name that reports the telemetry data. This will appear in the telemetry data as the `service.name` attribute.
  #[serde(default = "default_service_name")]
  pub service_name: String,
  /// A list of telemetry targets to send telemetry data to.
  ///
  /// The telemtry data is scoped per endpoint, and you can specify multiple targets if you need to export stats to multiple backends.
  pub targets: Vec<TelemetryTarget>,
}

fn default_service_name() -> String {
  "conductor".to_string()
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum TelemetryTarget {
  /// Sends telemetry data to `stdout` in a human-readable format.
  ///
  /// Use this source for debugging purposes, or if you want to pipe the telemetry data to another process.
  #[serde(rename = "stdout")]
  #[schemars(title = "stdout")]
  Stdout,
  /// Sends telemetry traces data to a [Zipkin](https://zipkin.io/) collector, using the HTTP protocol.
  ///
  /// To get started with Zipkin, use the following command to start the Zipkin collector and UI in your local machine, using Docker:
  ///
  /// `docker run -d -p 9411:9411 openzipkin/zipkin`
  #[serde(rename = "zipkin")]
  #[schemars(title = "zipkin")]
  Zipkin {
    #[serde(default = "default_zipkin_endpoint")]
    /// The Zipkin endpoint. Please use full URL endpoint format, e.g. `http://127.0.0.1:9411/api/v2/spans`.
    collector_endpoint: String,
  },
  /// Sends telemetry traces data to an [OpenTelemetry](https://opentelemetry.io/) backend, using the [OTLP protocol](https://opentelemetry.io/docs/specs/otel/protocol/).
  ///
  /// You can find [here a list backends that supports the OTLP format](https://github.com/magsther/awesome-opentelemetry#open-source).
  #[serde(rename = "otlp")]
  #[schemars(title = "Open Telemetry (OTLP)")]
  Otlp {
    /// The OTLP backend endpoint. The format is based on full URL, e.g. `http://localhost:7201`.
    endpoint: String,
    #[serde(default = "default_otlp_protocol")]
    /// The OTLP transport to use to export telemetry data.
    ///
    /// > ❗️ The gRPC transport is not supported on WASM runtime (CloudFlare Worker).
    protocol: OtlpProtcol,
    #[serde(
      deserialize_with = "humantime_serde::deserialize",
      serialize_with = "humantime_serde::serialize",
      default = "default_otlp_timeout"
    )]
    #[schemars(with = "String")]
    /// Export timeout. You can use the human-readable format in this field, e.g. `10s`.
    timeout: Duration,
    #[serde(default)]
    /// Whether to use gzip compression when sending telemetry data.
    ///
    /// Please verify your backend supports and enables `gzip` compression before enabling this option.
    gzip_compression: bool,
  },
  /// Sends telemetry traces data to a [Datadog](https://www.datadoghq.com/) agent (local or remote).
  ///
  /// To get started with Datadog, make sure you have a [Datadog agent running](https://docs.datadoghq.com/agent/?tab=source).
  #[serde(rename = "datadog")]
  #[schemars(title = "Datadog")]
  Datadog {
    /// The Datadog agent endpoint. The format is based on hostname and port only, e.g. `127.0.0.1:8126`.
    #[serde(default = "default_datadog_agent_endpoint")]
    agent_endpoint: SocketAddr,
  },
}

fn default_zipkin_endpoint() -> String {
  "http://127.0.0.1:9411/api/v2/spans".parse().unwrap()
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
  /// Uses GRPC with `tonic` to send telemetry data.
  #[schemars(title = "grpc")]
  #[serde(rename = "grpc")]
  Grpc,
  /// Uses HTTP with `http-proto` to send telemetry data.
  #[schemars(title = "http")]
  #[serde(rename = "http")]
  Http,
}

impl From<OtlpProtcol> for opentelemetry_otlp::Protocol {
  #[cfg(not(target_arch = "wasm32"))]
  fn from(value: OtlpProtcol) -> Self {
    match value {
      OtlpProtcol::Grpc => opentelemetry_otlp::Protocol::Grpc,
      OtlpProtcol::Http => opentelemetry_otlp::Protocol::HttpBinary,
    }
  }

  #[cfg(target_arch = "wasm32")]
  fn from(value: OtlpProtcol) -> Self {
    match value {
      OtlpProtcol::Grpc => {
        tracing::warn!("GRPC is not supported on WASM runtime. Using HTTP instead.");
        opentelemetry_otlp::Protocol::HttpBinary
      }
      OtlpProtcol::Http => opentelemetry_otlp::Protocol::HttpBinary,
    }
  }
}
