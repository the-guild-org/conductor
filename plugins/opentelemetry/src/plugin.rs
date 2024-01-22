use std::borrow::Cow;

use crate::config::{OpenTelemetryTarget, TelemetryPluginConfig};
use conductor_common::plugin::{CreatablePlugin, Plugin, PluginError};

use conductor_tracing::minitrace_mgr::MinitraceManager;
use minitrace::collector::Reporter;
use minitrace_opentelemetry::OpenTelemetryReporter;
use opentelemetry::trace::{SpanKind, TraceError};
use opentelemetry::{InstrumentationLibrary, KeyValue};
use opentelemetry_otlp::WithExportConfig;
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
  fn compose_reporter(
    service_name: &String,
    target: &OpenTelemetryTarget,
  ) -> Result<Box<dyn Reporter>, TraceError> {
    let reporter: Box<dyn Reporter> = match target {
      OpenTelemetryTarget::Jaeger { endpoint } => Box::new(minitrace_jaeger::JaegerReporter::new(
        endpoint.clone(),
        service_name,
      )?),
      OpenTelemetryTarget::Datadog { agent_endpoint } => {
        Box::new(minitrace_datadog::DatadogReporter::new(
          agent_endpoint.clone(),
          service_name,
          LIB_NAME,
          "web",
        ))
      }
      OpenTelemetryTarget::Otlp {
        endpoint,
        protocol,
        timeout,
        gzip_compression,
        ..
      } => {
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

  pub fn configure_tracing(
    &self,
    tenant_id: u32,
    tracing_manager: &mut MinitraceManager,
  ) -> Result<(), PluginError> {
    for target in &self.config.targets {
      let reporter = Self::compose_reporter(&self.config.service_name, &target)
        .map_err(|e| PluginError::InitError { source: e.into() })?;
      tracing_manager.add_reporter(tenant_id, reporter);
    }

    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for TelemetryPlugin {}
