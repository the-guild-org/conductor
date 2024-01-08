use std::sync::RwLock;

use opentelemetry::global;
use opentelemetry_sdk::{export::trace::SpanExporter, trace::SpanProcessor};

#[derive(Debug)]
pub struct AggregatingSpanProcessor {
  exporter: RwLock<Box<dyn SpanExporter>>,
  spans: RwLock<Vec<opentelemetry_sdk::export::trace::SpanData>>,
}

impl AggregatingSpanProcessor {
  pub fn new<E>(exporter: E) -> Self
  where
    E: SpanExporter + 'static,
  {
    Self {
      exporter: RwLock::new(Box::new(exporter)),
      spans: RwLock::new(Vec::new()),
    }
  }
}

impl SpanProcessor for AggregatingSpanProcessor {
  fn on_start(&self, _span: &mut opentelemetry_sdk::trace::Span, _cx: &opentelemetry::Context) {
    // Nothing to do here.
  }

  fn on_end(&self, span: opentelemetry_sdk::export::trace::SpanData) {
    if !span.span_context.is_sampled() {
      return;
    }

    match self.spans.write() {
      Ok(mut spans) => {
        spans.push(span);
      }
      Err(e) => {
        global::handle_error(e);
      }
    }
  }

  fn force_flush(&self) -> opentelemetry::trace::TraceResult<()> {
    match (self.spans.read(), self.exporter.write()) {
      (Ok(spans), Ok(mut exporter)) => {
        let export_fut = exporter.export(spans.to_vec());
        wasm_polyfills::spawn_local(async move {
          match export_fut.await {
            Ok(_) => {}
            Err(e) => global::handle_error(e),
          }
        });
      }
      _ => {
        panic!("Failed to lock spans or exporter")
      }
    }

    Ok(())
  }

  fn shutdown(&mut self) -> opentelemetry::trace::TraceResult<()> {
    Ok(())
  }
}
