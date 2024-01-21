use std::borrow::Cow;

use crate::config::{OpenTelemetryTarget, TelemetryPluginConfig};
// use crate::OpenTelemetryTracesLevel;
use conductor_common::plugin::{CreatablePlugin, Plugin, PluginError};
// use conductor_tracing::manager::{Registry, TracingManager};

use conductor_tracing::minitrace_mgr::MinitraceManager;
use opentelemetry::sdk::Resource;
use opentelemetry::trace::{SpanKind, TraceError};
use opentelemetry::{InstrumentationLibrary, KeyValue};
// use opentelemetry::{global, trace::TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
// use opentelemetry_sdk::propagation::TraceContextPropagator;
// use opentelemetry_sdk::trace::{self, SpanProcessor, TracerProvider};
// use opentelemetry_sdk::Resource;
// use tracing_subscriber::filter::ParseError;
// use tracing_subscriber::{EnvFilter, Layer};
use minitrace_opentelemetry::OpenTelemetryReporter;

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

impl TelemetryPlugin {
  // #[cfg(target_arch = "wasm32")]
  // fn build_span_processor(
  //   service_name: &str,
  //   target: &OpenTelemetryTarget,
  // ) -> Result<impl SpanProcessor, TraceError> {
  //   Ok(match target {
  //     OpenTelemetryTarget::Stdout { .. } => {
  //       conductor_tracing::wasm_span_processor::AggregatingSpanProcessor::new(
  //         opentelemetry_stdout::SpanExporter::default(),
  //       )
  //     }
  //     OpenTelemetryTarget::Jaeger { .. } => {
  //       return Err(TraceError::Other(
  //         anyhow::anyhow!(
  //           "Jaeger native is not supported in WASM runtime. Please use OTLP exporter instead."
  //         )
  //         .into(),
  //       ))
  //     }
  //     OpenTelemetryTarget::Otlp {
  //       endpoint,
  //       protocol,
  //       timeout,
  //       gzip_compression,
  //       ..
  //     } => {
  //       let exporter = opentelemetry_otlp::new_exporter()
  //         .http()
  //         .with_http_client(crate::wasm_runtime::WasmHttpClient::new())
  //         .with_endpoint(endpoint)
  //         .with_protocol(protocol.clone().into())
  //         .with_timeout(*timeout);

  //       if *gzip_compression {
  //         tracing::warn!("Gzip compression is not supported in WASM runtime. Ignoring.");
  //       }

  //       conductor_tracing::wasm_span_processor::AggregatingSpanProcessor::new(
  //         exporter.build_span_exporter()?,
  //       )
  //     }
  //     OpenTelemetryTarget::Zipkin { endpoint, .. } => {
  //       conductor_tracing::wasm_span_processor::AggregatingSpanProcessor::new(
  //         opentelemetry_zipkin::new_pipeline()
  //           .with_service_name(service_name)
  //           .with_collector_endpoint(endpoint)
  //           .with_http_client(crate::wasm_runtime::WasmHttpClient::new())
  //           .init_exporter()?,
  //       )
  //     }
  //     OpenTelemetryTarget::Datadog { endpoint, .. } => {
  //       conductor_tracing::wasm_span_processor::AggregatingSpanProcessor::new(
  //         opentelemetry_datadog::new_pipeline()
  //           .with_service_name(service_name)
  //           .with_api_version(opentelemetry_datadog::ApiVersion::Version05)
  //           .with_agent_endpoint(endpoint)
  //           .with_http_client(crate::wasm_runtime::WasmHttpClient::new())
  //           .build_exporter()?,
  //       )
  //     }
  //   })
  // }

  // #[cfg(not(target_arch = "wasm32"))]
  // fn build_span_processor(
  //   service_name: &str,
  //   target: &OpenTelemetryTarget,
  // ) -> Result<impl SpanProcessor, TraceError> {
  //   Ok(match target {
  //     OpenTelemetryTarget::Stdout { .. } => opentelemetry_sdk::trace::BatchSpanProcessor::builder(
  //       opentelemetry_stdout::SpanExporter::default(),
  //       opentelemetry_sdk::runtime::TokioCurrentThread,
  //     )
  //     .build(),
  //     OpenTelemetryTarget::Jaeger {
  //       endpoint,
  //       max_packet_size,
  //       batch_config,
  //       ..
  //     } => {
  //       tracing::warn!("Jaeger native exporter is deprecated. Please use OTLP exporter instead.");

  //       opentelemetry_sdk::trace::BatchSpanProcessor::builder(
  //         opentelemetry_jaeger::new_agent_pipeline()
  //           .with_endpoint(endpoint)
  //           .with_max_packet_size(*max_packet_size)
  //           .with_service_name(service_name)
  //           .build_async_agent_exporter(opentelemetry_sdk::runtime::TokioCurrentThread)?,
  //         opentelemetry_sdk::runtime::TokioCurrentThread,
  //       )
  //       .with_batch_config(batch_config.into())
  //       .build()
  //     }
  //     OpenTelemetryTarget::Otlp {
  //       endpoint,
  //       protocol,
  //       timeout,
  //       gzip_compression,
  //       batch_config,
  //       ..
  //     } => {
  //       let mut exporter = opentelemetry_otlp::new_exporter()
  //         .tonic()
  //         .with_endpoint(endpoint)
  //         .with_protocol(protocol.clone().into())
  //         .with_timeout(*timeout);

  //       if *gzip_compression {
  //         exporter = exporter.with_compression(opentelemetry_otlp::Compression::Gzip);
  //       }

  //       opentelemetry_sdk::trace::BatchSpanProcessor::builder(
  //         exporter.build_span_exporter()?,
  //         opentelemetry_sdk::runtime::TokioCurrentThread,
  //       )
  //       .with_batch_config(batch_config.into())
  //       .build()
  //     }
  //     OpenTelemetryTarget::Zipkin {
  //       endpoint,
  //       batch_config,
  //       ..
  //     } => opentelemetry_sdk::trace::BatchSpanProcessor::builder(
  //       opentelemetry_zipkin::new_pipeline()
  //         .with_service_name(service_name)
  //         .with_collector_endpoint(endpoint)
  //         .init_exporter()?,
  //       opentelemetry_sdk::runtime::TokioCurrentThread,
  //     )
  //     .with_batch_config(batch_config.into())
  //     .build(),
  //     OpenTelemetryTarget::Datadog {
  //       endpoint,
  //       batch_config,
  //       ..
  //     } => opentelemetry_sdk::trace::BatchSpanProcessor::builder(
  //       opentelemetry_datadog::new_pipeline()
  //         .with_service_name(service_name)
  //         .with_api_version(opentelemetry_datadog::ApiVersion::Version05)
  //         .with_agent_endpoint(endpoint)
  //         .with_http_client(wasm_polyfills::create_http_client().build().unwrap())
  //         .build_exporter()?,
  //       opentelemetry_sdk::runtime::TokioCurrentThread,
  //     )
  //     .with_batch_config(batch_config.into())
  //     .build(),
  //   })
  // }

  // fn compose_filter(endpoint: &str, level: &str) -> Result<EnvFilter, ParseError> {
  //   let directive = format!("[{{endpoint=\"{}\"}}]={}", endpoint, level);
  //   EnvFilter::try_new(directive)
  // }

  fn compose_reporter(
    service_name: &String,
    target: &OpenTelemetryTarget,
  ) -> Result<OpenTelemetryReporter, TraceError> {
    let exporter = match target {
      OpenTelemetryTarget::Otlp {
        endpoint,
        protocol,
        timeout,
        gzip_compression,
        batch_config,
      } => {
        // let mut exporter = opentelemetry_otlp::new_exporter()
        //   .tonic()
        //   .with_endpoint(endpoint)
        //   .with_protocol(protocol.clone().into())
        //   .with_timeout(*timeout);

        // if *gzip_compression {
        //   exporter = exporter.with_compression(opentelemetry_otlp::Compression::Gzip);
        // }

        opentelemetry_otlp::SpanExporter::new_tonic(
          opentelemetry_otlp::ExportConfig {
            endpoint: endpoint.clone(),
            protocol: protocol.clone().into(),
            timeout: *timeout,
          },
          opentelemetry_otlp::TonicConfig::default(),
        )
        .unwrap()
      }
      _ => todo!(),
    };

    let reporter = OpenTelemetryReporter::new(
      exporter,
      SpanKind::Server,
      Cow::Owned(Resource::new([KeyValue::new(
        "service.name",
        service_name.clone(),
      )])),
      InstrumentationLibrary::new(
        "conductor",
        Some(env!("CARGO_PKG_VERSION")),
        None::<&'static str>,
        None,
      ),
    );

    Ok(reporter)
  }

  pub fn configure_tracing(
    &self,
    endpoint_identifier: &str,
    tracing_manager: &mut MinitraceManager,
  ) -> Result<(), PluginError> {
    // global::set_text_map_propagator(TraceContextPropagator::new());
    // opentelemetry::global::set_error_handler(|e| {
    //   tracing::error!("Failed to export telemetry data: {:?}", e);
    // })
    // .unwrap();

    // for target in self.config.targets.clone().into_iter() {
    //   let filter = Self::compose_filter(endpoint_identifier, &target.level().to_string())
    //     .map_err(|e| PluginError::InitError { source: e.into() })?;
    //   let span_processor = Self::build_span_processor(&self.config.service_name, &target)
    //     .map_err(|e| PluginError::InitError { source: e.into() })?;
    //   let tracer_provider = TracerProvider::builder()
    //     .with_config(
    //       trace::config().with_resource(Resource::new(vec![KeyValue::new(
    //         "service.name",
    //         self.config.service_name.clone(),
    //       )])),
    //     )
    //     .with_span_processor(span_processor)
    //     .build();
    //   let tracer = tracer_provider.tracer(self.config.service_name.clone());

    //   let debug_info = match target.level() {
    //     OpenTelemetryTracesLevel::Debug => true,
    //     _ => false,
    //   };

    //   let layer = tracing_opentelemetry::layer::<Registry>()
    //     .with_location(debug_info)
    //     .with_threads(debug_info)
    //     .with_tracked_inactivity(debug_info)
    //     .with_tracer(tracer)
    //     .with_filter(filter)
    //     .boxed();

    // tracing_manager.register_provider(tracer_provider);
    // tracing_manager.add_tracing_layer(layer);
    // }

    for target in &self.config.targets {
      let reporter = Self::compose_reporter(&self.config.service_name, &target)
        .map_err(|e| PluginError::InitError { source: e.into() })?;
      tracing_manager.add_reporter(endpoint_identifier.to_string(), Box::new(reporter));
    }

    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl Plugin for TelemetryPlugin {}
