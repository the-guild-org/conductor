use minitrace::collector::Reporter;

use crate::{
  reporters::TracingReporter, routed_reporter::RoutedReporter, trace_id::extract_tenant_id,
};
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

#[derive(Default)]
pub struct MinitraceManager {
  reporters: HashMap<u32, Arc<Mutex<TracingReporter>>>,
}

impl std::fmt::Debug for MinitraceManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MinitraceManager").finish()
  }
}

impl MinitraceManager {
  pub fn add_reporter(&mut self, tenant_id: u32, reporter: TracingReporter) {
    self
      .reporters
      .insert(tenant_id, Arc::new(Mutex::new(reporter)));
  }

  pub fn build_root_reporter(&self) -> impl Reporter {
    let mut routed_reporter = RoutedReporter::new(|span| Some(extract_tenant_id(span.trace_id)));

    for (tenant_id, reporter) in &self.reporters {
      routed_reporter = routed_reporter.with_reporter(*tenant_id, reporter.clone());
    }

    routed_reporter
  }

  #[allow(clippy::await_holding_lock)]
  pub async fn shutdown(self) {
    tracing::info!("Shutting down tracing reporters...");
    minitrace::flush();

    for (_, reporter) in self.reporters {
      let mut reporter = reporter
        .lock()
        .expect("failed to acquire lock for tracing reporter");

      reporter.flush().await
    }
  }
}
