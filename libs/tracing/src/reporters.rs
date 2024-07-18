use fastrace::collector::{Reporter as FastraceSyncReporter, SpanRecord};

#[async_trait::async_trait(?Send)]
pub trait AsyncReporter: Send + 'static {
  async fn flush(&mut self, spans: &[SpanRecord]);
}

pub struct AggregatingReporter {
  collected_spans: Vec<SpanRecord>,
  reporter: Box<dyn AsyncReporter>,
}

impl AggregatingReporter {
  pub fn new(reporter: Box<dyn AsyncReporter>) -> Self {
    Self {
      collected_spans: Vec::new(),
      reporter,
    }
  }

  pub async fn flush(&mut self) {
    self.reporter.flush(&self.collected_spans).await;
  }
}

impl FastraceSyncReporter for AggregatingReporter {
  fn report(&mut self, spans: &[SpanRecord]) {
    self.collected_spans.extend_from_slice(spans);
  }
}

pub enum TracingReporter {
  // A simple wrapper around a generic Reporter created from fastrace package.
  Simple(Box<dyn FastraceSyncReporter>),
  // A special reporter that aggregates the spans in memory, and can later ship the spans on demand.
  // This is a workaround that collects traces in-memory, and later ships them asynchronously, on a WASM runtime.
  Aggregating(AggregatingReporter),
}

impl TracingReporter {
  pub fn report(&mut self, spans: &[SpanRecord]) {
    match self {
      TracingReporter::Aggregating(reporter) => reporter.report(spans),
      TracingReporter::Simple(reporter) => reporter.report(spans),
    }
  }

  pub async fn flush(&mut self) {
    // Only the AggregatingReporter needs to flush the spans at this point.
    if let TracingReporter::Aggregating(reporter) = self {
      reporter.flush().await;
    }
  }
}
