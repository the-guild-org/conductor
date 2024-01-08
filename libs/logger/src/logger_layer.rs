use crate::config::LoggerConfigFormat;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;
use tracing_subscriber::{
  fmt::{self, time::UtcTime},
  Layer,
};

#[cfg(target_arch = "wasm32")]
pub fn build_logger(
  format: &LoggerConfigFormat,
  filter: &str,
  print_performance_info: bool,
) -> Result<Box<dyn Layer<Registry> + Send + Sync>, tracing_subscriber::filter::ParseError> {
  // TL;DR: WASM logger config
  // ANSI is not supported in all log processors, so we can disable it.
  // We are using a custom timer because std::time is not available in WASM.
  // Writer is configured to MakeWebConsoleWriter so all logs will go to JS `console`.

  let timer = UtcTime::rfc_3339();
  let filter = EnvFilter::try_new(filter)?;

  if print_performance_info {
    println!("Logger flag \"print_performance_info\" is not supported in WASM runtime, ignoring.");
  }

  Ok(match format {
    LoggerConfigFormat::Json => fmt::Layer::<Registry>::default()
      .json()
      .with_ansi(false)
      .with_timer(timer)
      .with_writer(tracing_web::MakeWebConsoleWriter::new())
      .with_filter(filter)
      .boxed(),
    LoggerConfigFormat::Pretty => fmt::Layer::<Registry>::default()
      .pretty()
      .with_timer(timer)
      .with_writer(tracing_web::MakeWebConsoleWriter::new())
      .with_filter(filter)
      .boxed(),
    LoggerConfigFormat::Compact => fmt::Layer::<Registry>::default()
      .compact()
      .with_timer(timer)
      .with_writer(tracing_web::MakeWebConsoleWriter::new())
      .with_filter(filter)
      .boxed(),
  })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn build_logger(
  format: &LoggerConfigFormat,
  filter: &str,
  print_performance_info: bool,
) -> Result<Box<dyn Layer<Registry> + Send + Sync>, tracing_subscriber::filter::ParseError> {
  let timer = UtcTime::rfc_3339();
  let filter = EnvFilter::try_new(filter)?;
  let performance_spans = match print_performance_info {
    true => tracing_subscriber::fmt::format::FmtSpan::CLOSE,
    false => tracing_subscriber::fmt::format::FmtSpan::NONE,
  };

  Ok(match format {
    LoggerConfigFormat::Json => fmt::Layer::<Registry>::default()
      .json()
      .with_timer(timer)
      .with_span_events(performance_spans)
      .with_filter(filter)
      .boxed(),
    LoggerConfigFormat::Pretty => fmt::Layer::<Registry>::default()
      .pretty()
      .with_timer(timer)
      .with_span_events(performance_spans)
      .with_filter(filter)
      .boxed(),
    LoggerConfigFormat::Compact => fmt::Layer::<Registry>::default()
      .compact()
      .with_timer(timer)
      .with_span_events(performance_spans)
      .with_filter(filter)
      .boxed(),
  })
}
