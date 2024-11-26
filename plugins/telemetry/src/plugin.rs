use crate::config::{TelemetryPluginConfig, TelemetryTarget};
use conductor_common::plugin::{CreatablePlugin, Plugin, PluginError};
use conductor_tracing::fastrace_mgr::FastraceManager;
use conductor_tracing::reporters::TracingReporter;
use opentelemetry::trace::SpanKind;
use opentelemetry::trace::TraceError;
use opentelemetry::InstrumentationScope;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::export::trace::SpanExporter;
use opentelemetry_sdk::Resource;

#[derive(Debug)]
pub struct TelemetryPlugin {
  config: TelemetryPluginConfig,
}

#[async_trait::async_trait(?Send)]
impl CreatablePlugin for TelemetryPlugin {
  type Config = TelemetryPluginConfig;

  async fn create(config: Self::Config) -> Result<Box<TelemetryPlugin>, PluginError> {
    Ok(Box::new(Self { config }))
  }
}

static LIB_NAME: &str = "conductor";

impl TelemetryPlugin {
  #[cfg(target_arch = "wasm32")]
  fn compose_reporter(
    service_name: &String,
    target: &TelemetryTarget,
  ) -> Result<TracingReporter, TraceError> {
    use crate::wasm_reporter::console_reporter::WasmConsoleReporter;
    use crate::wasm_reporter::datadog_reporter::WasmDatadogReporter;
    use crate::wasm_reporter::otlp_reporter::{WasmOtlpReporter, WasmTracingHttpClient};
    use conductor_tracing::reporters::AggregatingReporter;

    let reporter: TracingReporter = match target {
      TelemetryTarget::Stdout => TracingReporter::Simple(Box::new(WasmConsoleReporter)),
      TelemetryTarget::Jaeger { .. } => {
        return Err(TraceError::Other(
            "The \"jaeger\" target is not supported on WASM runtime. Please use the \"otlp\" target instead. See: https://opentelemetry.io/blog/2022/jaeger-native-otlp/".into(),
        ))
      },
      TelemetryTarget::Zipkin { collector_endpoint } => {
        let (lib, resource) = Self::build_otlp_identifiers(service_name.clone());

        let exporter = opentelemetry_zipkin::ZipkinPipelineBuilder::default()
          .with_service_name(service_name)
          .with_http_client(WasmTracingHttpClient)
          .with_collector_endpoint(collector_endpoint)
          .init_exporter()?;

          let reporter = Box::new(WasmOtlpReporter::new(
            exporter,
            SpanKind::Server,
            resource,
            lib,
          ));

          TracingReporter::Aggregating(AggregatingReporter::new(reporter))
      }
      TelemetryTarget::Datadog { agent_endpoint } => TracingReporter::Aggregating(AggregatingReporter::new(Box::new(WasmDatadogReporter::new(agent_endpoint, service_name, LIB_NAME, "web")))),
      TelemetryTarget::Otlp { endpoint,
        protocol,
        timeout,
        gzip_compression } => {
        let (lib, resource) = Self::build_otlp_identifiers(service_name.clone());
        let exporter = opentelemetry_otlp::new_exporter()
          .http()
          .with_http_client(WasmTracingHttpClient)
          .with_endpoint(endpoint)
          .with_protocol(protocol.clone().into())
          .with_timeout(*timeout);

        if *gzip_compression {
          tracing::warn!("Gzip compression is not supported on WASM runtime. Ignoring.");
        }

        let reporter = Box::new(WasmOtlpReporter::new(
          exporter.build_span_exporter()?,
          SpanKind::Server,
          resource,
          lib,
        ));

          TracingReporter::Aggregating(AggregatingReporter::new(reporter))
        },
    };

    Ok(reporter)
  }

  fn build_otlp_scope(service_name: String) -> (InstrumentationScope, Resource) {
    let scope = InstrumentationScope::builder(LIB_NAME).build();
    let resource = Resource::new(vec![KeyValue::new("service.name", service_name)]);

    (scope, resource)
  }

  #[cfg(not(target_arch = "wasm32"))]
  fn compose_reporter(
    service_name: &String,
    target: &TelemetryTarget,
  ) -> Result<TracingReporter, TraceError> {
    use fastrace::collector::ConsoleReporter;
    use fastrace::collector::Reporter;
    use opentelemetry_otlp::WithHttpConfig;
    use opentelemetry_otlp::WithTonicConfig;

    use crate::config::OtlpProtcol;
    use crate::reporter::open_telemetry::OpenTelemetryReporter;

    let reporter: Box<dyn Reporter> = match target {
      TelemetryTarget::Stdout => Box::new(ConsoleReporter),
      TelemetryTarget::Zipkin { collector_endpoint } => {
        let (scope, _) = Self::build_otlp_scope(service_name.clone());

        let exporter = opentelemetry_zipkin::ZipkinPipelineBuilder::default()
          .with_service_name(service_name)
          .with_http_client(reqwest::Client::new())
          .with_collector_endpoint(collector_endpoint)
          .init_exporter()?;

        Box::new(OpenTelemetryReporter::new(
          exporter,
          SpanKind::Server,
          scope,
        ))
      }
      TelemetryTarget::Datadog { agent_endpoint } => Box::new(
        fastrace_datadog::DatadogReporter::new(*agent_endpoint, service_name, LIB_NAME, "web"),
      ),
      TelemetryTarget::Otlp {
        endpoint,
        protocol,
        timeout,
        gzip_compression,
      } => {
        let mut exporter = match protocol {
          OtlpProtcol::Http => {
            let builder = opentelemetry_otlp::SpanExporter::builder()
              .with_http()
              .with_http_client(reqwest::Client::new())
              .with_endpoint(endpoint)
              .with_protocol(protocol.clone().into())
              .with_timeout(*timeout);

            if *gzip_compression {
              tracing::warn!("Gzip compression is not supported on HTTP protocol. Ignoring.");
            }

            builder.build()?
          }
          OtlpProtcol::Grpc => {
            let mut builder = opentelemetry_otlp::SpanExporter::builder()
              .with_tonic()
              .with_endpoint(endpoint)
              .with_protocol(protocol.clone().into())
              .with_timeout(*timeout);

            if *gzip_compression {
              builder = builder.with_compression(opentelemetry_otlp::Compression::Gzip);
            }

            builder.build()?
          }
        };

        let (scope, resource) = Self::build_otlp_scope(service_name.clone());

        exporter.set_resource(&resource);

        Box::new(OpenTelemetryReporter::new(
          exporter,
          SpanKind::Server,
          scope,
        ))
      }
    };

    Ok(TracingReporter::Simple(reporter))
  }

  #[cfg(feature = "test_utils")]
  pub fn configure_tracing_for_test(
    &self,
    tenant_id: u32,
    reporter: TracingReporter,
    tracing_manager: &mut FastraceManager,
  ) {
    tracing_manager.add_reporter(tenant_id, reporter);
  }

  pub fn configure_tracing(
    &self,
    tenant_id: u32,
    tracing_manager: &mut FastraceManager,
  ) -> Result<(), PluginError> {
    for target in &self.config.targets {
      let reporter = Self::compose_reporter(&self.config.service_name, target)
        .map_err(|e| PluginError::InitError { source: e.into() })?;
      tracing_manager.add_reporter(tenant_id, reporter);
    }

    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for TelemetryPlugin {}
