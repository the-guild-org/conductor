use fastrace::collector::{Reporter, SpanRecord};

pub struct WasmConsoleReporter;

impl Reporter for WasmConsoleReporter {
  fn report(&mut self, spans: &[SpanRecord]) {
    for span in spans {
      tracing::info!("{:#?}", span);
    }
  }
}
