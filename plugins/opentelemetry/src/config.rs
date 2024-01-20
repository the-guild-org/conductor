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
pub enum OpenTelemetryTracesLevel {
  #[serde(rename = "info")]
  #[schemars(title = "info")]
  Info,
  #[serde(rename = "debug")]
  #[schemars(title = "debug")]
  Debug,
}

impl std::fmt::Display for OpenTelemetryTracesLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      OpenTelemetryTracesLevel::Info => write!(f, "info"),
      OpenTelemetryTracesLevel::Debug => write!(f, "debug"),
    }
  }
}

impl Default for OpenTelemetryTracesLevel {
  fn default() -> Self {
    OpenTelemetryTracesLevel::Info
  }
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum OpenTelemetryTarget {
  #[serde(rename = "stdout")]
  #[schemars(title = "stdout")]
  Stdout {
    #[serde(default)]
    level: OpenTelemetryTracesLevel,
  },
  #[serde(rename = "jaeger")]
  #[schemars(title = "jaeger")]
  Jaeger {
    #[serde(default = "default_jaeger_endpoint")]
    endpoint: String,
    #[serde(default = "default_jaeger_max_packet_size")]
    max_packet_size: usize,
    #[serde(default)]
    level: OpenTelemetryTracesLevel,
    #[serde(default = "default_batch_config")]
    batch_config: OpenTelemetryBatchExportConfig,
  },
  #[serde(rename = "otlp")]
  #[schemars(title = "otlp")]
  Otlp {
    #[serde(default)]
    level: OpenTelemetryTracesLevel,
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
    #[serde(default = "default_batch_config")]
    batch_config: OpenTelemetryBatchExportConfig,
  },
  #[serde(rename = "zipkin")]
  #[schemars(title = "zipkin")]
  // TODO: docs, mention /api/v2/spans path
  Zipkin {
    #[serde(default)]
    level: OpenTelemetryTracesLevel,
    endpoint: String,
    #[serde(default = "default_batch_config")]
    batch_config: OpenTelemetryBatchExportConfig,
  },
  #[serde(rename = "datadog")]
  #[schemars(title = "datadog")]
  Datadog {
    #[serde(default)]
    level: OpenTelemetryTracesLevel,
    #[serde(default = "default_datadog_endpoint")]
    endpoint: String,
    #[serde(default = "default_batch_config")]
    batch_config: OpenTelemetryBatchExportConfig,
  },
}

impl OpenTelemetryTarget {
  pub fn level(&self) -> &OpenTelemetryTracesLevel {
    match self {
      OpenTelemetryTarget::Stdout { level } => level,
      OpenTelemetryTarget::Jaeger { level, .. } => level,
      OpenTelemetryTarget::Otlp { level, .. } => level,
      OpenTelemetryTarget::Zipkin { level, .. } => level,
      OpenTelemetryTarget::Datadog { level, .. } => level,
    }
  }
}

fn default_batch_config() -> OpenTelemetryBatchExportConfig {
  OpenTelemetryBatchExportConfig {
    max_queue_size: default_max_queue_size(),
    scheduled_delay: default_scheduled_delay(),
    max_export_batch_size: default_max_export_batch_size(),
    max_export_timeout: default_max_export_timeout(),
    max_concurrent_exports: default_max_concurrent_exports(),
  }
}

// impl From<&OpenTelemetryBatchExportConfig> for BatchConfig {
//   fn from(value: &OpenTelemetryBatchExportConfig) -> Self {
//     BatchConfig::default()
//       .with_max_queue_size(value.max_queue_size)
//       .with_scheduled_delay(value.scheduled_delay)
//       .with_max_export_batch_size(value.max_export_batch_size)
//       .with_max_export_timeout(value.max_export_timeout)
//       .with_max_concurrent_exports(value.max_concurrent_exports)
//   }
// }

fn default_max_queue_size() -> usize {
  2048
}

fn default_scheduled_delay() -> Duration {
  Duration::from_secs(5)
}

fn default_max_export_timeout() -> Duration {
  Duration::from_secs(30)
}

fn default_max_export_batch_size() -> usize {
  512
}

fn default_max_concurrent_exports() -> usize {
  1
}

#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
pub struct OpenTelemetryBatchExportConfig {
  /// The maximum queue size to buffer spans for delayed processing. If the
  /// queue gets full it drops the spans.
  #[serde(default = "default_max_queue_size")]
  max_queue_size: usize,

  /// The delay interval in milliseconds between two consecutive processing
  /// of batches.
  #[serde(default = "default_scheduled_delay")]
  scheduled_delay: Duration,

  /// The maximum number of spans to process in a single batch. If there are
  /// more than one batch worth of spans then it processes multiple batches
  /// of spans one batch after the other without any delay.
  #[serde(default = "default_max_export_batch_size")]
  max_export_batch_size: usize,

  /// The maximum duration to export a batch of data.
  #[serde(default = "default_max_export_timeout")]
  max_export_timeout: Duration,

  /// Maximum number of concurrent exports
  ///
  /// Limits the number of spawned tasks for exports and thus memory consumed
  /// by an exporter.
  /// A value of 1 will cause exports to be performed synchronously on the exporter task.
  #[serde(default = "default_max_concurrent_exports")]
  max_concurrent_exports: usize,
}

fn default_otlp_protocol() -> OtlpProtcol {
  OtlpProtcol::Grpc
}

fn default_otlp_timeout() -> Duration {
  Duration::from_secs(10)
}

fn default_datadog_endpoint() -> String {
  "http://127.0.0.1:8126".to_string()
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

fn default_jaeger_max_packet_size() -> usize {
  65_000
}

fn default_jaeger_endpoint() -> String {
  "127.0.0.1:6831".to_string()
}

// fn graphiql_example() -> JsonSchemaExample<GraphiQLPluginConfig> {
//   JsonSchemaExample {
//     metadata: JsonSchemaExampleMetadata::new("Enable GraphiQL", None),
//     wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
//       name: "graphiql".to_string(),
//     }),
//     example: GraphiQLPluginConfig {
//       headers_editor_enabled: Default::default(),
//     },
//   }
// }

// // At some point, it might be worth supporting more options. see:
// // https://github.com/dotansimha/graphql-yoga/blob/main/packages/graphiql/src/YogaGraphiQL.tsx#L35
// #[derive(Deserialize, Serialize, Debug, Clone)]
// pub struct GraphiQLSource {
//   pub endpoint: String,
//   pub query: String,
//   #[serde(rename = "isHeadersEditorEnabled")]
//   pub headers_editor_enabled: bool,
// }
