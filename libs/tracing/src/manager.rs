use opentelemetry_sdk::trace::TracerProvider;
use tracing::Subscriber;
use tracing_subscriber::filter::ParseError;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{
  fmt::{self, time::UtcTime},
  registry::LookupSpan,
  reload::{self, Handle},
  Layer,
};

pub use tracing_subscriber::Registry;

use crate::config::LoggerConfigFormat;

pub struct TracingManagerImpl<S>
where
  S: Subscriber,
  for<'a> S: LookupSpan<'a>,
{
  dynamic_handler: Handle<Vec<Box<dyn Layer<S> + Send + Sync>>, S>,
  providers: Vec<TracerProvider>,
}

impl<S> std::fmt::Debug for TracingManagerImpl<S>
where
  S: Subscriber,
  for<'a> S: LookupSpan<'a>,
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TracingManagerImpl").finish()
  }
}

impl<S> TracingManagerImpl<S>
where
  S: Subscriber + Send + Sync,
  for<'a> S: LookupSpan<'a>,
{
  pub fn new(
    format: &LoggerConfigFormat,
    filter: &str,
    print_performance_info: bool,
  ) -> Result<(Self, Box<(dyn Layer<S> + Send + Sync + 'static)>), ParseError> {
    let logger_layer = TracingManagerImpl::make_root_layer(format, filter, print_performance_info)?;
    let (dynamic_layer, dynamic_handler) =
      reload::Layer::<Vec<Box<dyn Layer<S> + Send + Sync>>, S>::new(vec![]);

    Ok((
      Self {
        dynamic_handler,
        providers: vec![],
      },
      Box::new(vec![dynamic_layer.boxed(), logger_layer]),
    ))
  }

  #[cfg(target_arch = "wasm32")]
  fn make_root_layer(
    format: &LoggerConfigFormat,
    filter: &str,
    print_performance_info: bool,
  ) -> Result<Box<dyn Layer<S> + Send + Sync>, tracing_subscriber::filter::ParseError> {
    // TL;DR: WASM logger config
    // ANSI is not supported in all log processors, so we can disable it.
    // We are using a custom timer because std::time is not available in WASM.
    // Writer is configured to MakeWebConsoleWriter so all logs will go to JS `console`.

    let timer = UtcTime::rfc_3339();
    let filter = EnvFilter::try_new(filter)?;

    if print_performance_info {
      println!(
        "Logger flag \"print_performance_info\" is not supported in WASM runtime, ignoring."
      );
    }

    Ok(match format {
      LoggerConfigFormat::Json => fmt::Layer::<S>::default()
        .json()
        .with_ansi(false)
        .with_timer(timer)
        .with_writer(tracing_web::MakeWebConsoleWriter::new())
        .with_filter(filter)
        .boxed(),
      LoggerConfigFormat::Pretty => fmt::Layer::<S>::default()
        .pretty()
        .with_timer(timer)
        .with_writer(tracing_web::MakeWebConsoleWriter::new())
        .with_filter(filter)
        .boxed(),
      LoggerConfigFormat::Compact => fmt::Layer::<S>::default()
        .compact()
        .with_timer(timer)
        .with_writer(tracing_web::MakeWebConsoleWriter::new())
        .with_filter(filter)
        .boxed(),
    })
  }

  #[cfg(not(target_arch = "wasm32"))]
  fn make_root_layer(
    format: &LoggerConfigFormat,
    filter: &str,
    print_performance_info: bool,
  ) -> Result<Box<dyn Layer<S> + Send + Sync>, tracing_subscriber::filter::ParseError> {
    let timer = UtcTime::rfc_3339();
    let filter = EnvFilter::try_new(filter)?;
    let performance_spans = match print_performance_info {
      true => tracing_subscriber::fmt::format::FmtSpan::CLOSE,
      false => tracing_subscriber::fmt::format::FmtSpan::NONE,
    };

    Ok(match format {
      LoggerConfigFormat::Json => fmt::Layer::<S>::default()
        .json()
        .with_timer(timer)
        .with_span_events(performance_spans)
        .with_filter(filter)
        .boxed(),
      LoggerConfigFormat::Pretty => fmt::Layer::<S>::default()
        .pretty()
        .with_timer(timer)
        .with_span_events(performance_spans)
        .with_filter(filter)
        .boxed(),
      LoggerConfigFormat::Compact => fmt::Layer::<S>::default()
        .compact()
        .with_timer(timer)
        .with_span_events(performance_spans)
        .with_filter(filter)
        .boxed(),
    })
  }

  #[cfg(target_arch = "wasm32")]
  pub async fn shutdown(self) {
    for provider in self.providers {
      provider.force_flush();
    }

    opentelemetry::global::shutdown_tracer_provider();
  }

  #[cfg(not(target_arch = "wasm32"))]
  pub async fn shutdown(self) {
    tokio::runtime::Handle::current()
      .spawn_blocking(move || {
        opentelemetry::global::shutdown_tracer_provider();
      })
      .await
      .unwrap();
  }

  pub fn register_provider(&mut self, provider: TracerProvider) {
    self.providers.push(provider);
  }

  pub fn add_tracing_layer(&mut self, layer: Box<dyn Layer<S> + Send + Sync>) {
    self
      .dynamic_handler
      .modify(|layers| {
        layers.push(layer);
      })
      .expect("failed to add tracing layer")
  }
}

pub type TracingManager = TracingManagerImpl<Registry>;
