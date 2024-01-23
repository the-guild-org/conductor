use crate::config::{TelemetryPluginConfig, TelemetryTarget};
use conductor_common::plugin::{CreatablePlugin, Plugin, PluginError};

use conductor_tracing::minitrace_mgr::MinitraceManager;
use minitrace::collector::Reporter;
use opentelemetry::trace::TraceError;

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

#[cfg(not(target_arch = "wasm32"))]
static LIB_NAME: &str = "conductor";

impl TelemetryPlugin {
  #[cfg(target_arch = "wasm32")]
  fn compose_reporter(
    _service_name: &String,
    _target: &TelemetryTarget,
  ) -> Result<Box<dyn Reporter>, TraceError> {
    Err(TraceError::Other(
      "plugin is not supported in this runtime".into(),
    ))
  }

  #[cfg(not(target_arch = "wasm32"))]
  fn compose_reporter(
    service_name: &String,
    target: &TelemetryTarget,
  ) -> Result<Box<dyn Reporter>, TraceError> {
    use minitrace::collector::ConsoleReporter;
    use minitrace_opentelemetry::OpenTelemetryReporter;

    let reporter: Box<dyn Reporter> = match target {
      TelemetryTarget::Stdout => Box::new(ConsoleReporter),
      TelemetryTarget::Jaeger { endpoint } => Box::new(minitrace_jaeger::JaegerReporter::new(
        *endpoint,
        service_name,
      )?),
      TelemetryTarget::Datadog { agent_endpoint } => Box::new(
        minitrace_datadog::DatadogReporter::new(*agent_endpoint, service_name, LIB_NAME, "web"),
      ),
      TelemetryTarget::Otlp {
        endpoint,
        protocol,
        timeout,
        gzip_compression,
      } => {
        use opentelemetry::trace::SpanKind;
        use opentelemetry::{InstrumentationLibrary, KeyValue};
        use opentelemetry_otlp::WithExportConfig;
        use opentelemetry_sdk::Resource;
        use std::borrow::Cow;

        let lib =
          InstrumentationLibrary::new(LIB_NAME, None::<&'static str>, None::<&'static str>, None);
        let resource = Cow::Owned(Resource::new([KeyValue::new(
          "service.name",
          service_name.clone(),
        )]));

        let mut exporter = opentelemetry_otlp::new_exporter()
          .tonic()
          .with_endpoint(endpoint)
          .with_protocol(protocol.clone().into())
          .with_timeout(*timeout);

        if *gzip_compression {
          exporter = exporter.with_compression(opentelemetry_otlp::Compression::Gzip);
        }

        Box::new(OpenTelemetryReporter::new(
          exporter.build_span_exporter()?,
          SpanKind::Server,
          resource,
          lib,
        ))
      }
    };

    Ok(reporter)
  }

  #[cfg(feature = "test_utils")]
  pub fn configure_tracing_for_test(
    &self,
    tenant_id: u32,
    reporter: Box<dyn Reporter>,
    tracing_manager: &mut MinitraceManager,
  ) {
    tracing_manager.add_reporter(tenant_id, reporter);
  }

  pub fn configure_tracing(
    &self,
    tenant_id: u32,
    tracing_manager: &mut MinitraceManager,
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
