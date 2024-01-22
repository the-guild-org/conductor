use std::collections::HashMap;

use minitrace::collector::{Reporter, SpanRecord, TraceId};

pub struct MinitraceManager {
  reporters: HashMap<u32, Box<dyn Reporter>>,
}

impl std::fmt::Debug for MinitraceManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MinitraceManager").finish()
  }
}

impl MinitraceManager {
  pub fn new() -> Self {
    Self {
      reporters: HashMap::new(),
    }
  }

  pub fn add_reporter(&mut self, tenant_id: u32, reporter: Box<dyn Reporter>) {
    self.reporters.insert(tenant_id, reporter);
  }

  pub fn generate_trace_id(tenant_id: u32) -> TraceId {
    let uniq: u32 = rand::random();

    TraceId(((tenant_id as u128) << 32) | (uniq as u128))
  }

  pub fn extract_tenant_id(trace_id: TraceId) -> u32 {
    (trace_id.0 >> 32) as u32
  }

  pub fn build_reporter(self) -> impl Reporter {
    let mut routed_reporter =
      RoutedReporter::new(|span| Some(Self::extract_tenant_id(span.trace_id)));

    for (tenant_id, reporter) in self.reporters {
      routed_reporter = routed_reporter.with_reporter(tenant_id, reporter);
    }

    routed_reporter
  }
}

type RouterFn = fn(&SpanRecord) -> Option<u32>;

struct RoutedReporter {
  reporters: HashMap<u32, Box<dyn Reporter>>,
  router_fn: RouterFn,
}

impl RoutedReporter {
  pub fn new(router_fn: RouterFn) -> Self {
    Self {
      reporters: HashMap::new(),
      router_fn,
    }
  }

  pub fn with_reporter(mut self, tenant_id: u32, reporter: Box<dyn Reporter>) -> Self {
    self.reporters.insert(tenant_id, reporter);

    self
  }
}

impl Reporter for RoutedReporter {
  fn report(&mut self, spans: &[SpanRecord]) {
    let mut chunks: HashMap<u32, Vec<SpanRecord>> = HashMap::new();

    for span in spans {
      if let Some(key) = (self.router_fn)(span) {
        let chunk = chunks.entry(key).or_insert_with(Vec::new);
        chunk.push(span.clone());
      } else {
        tracing::warn!("no key for span: {:?}, dropping span", span);
      }
    }

    for (key, chunk) in chunks {
      if let Some(reporter) = self.reporters.get_mut(&key) {
        reporter.report(chunk.as_slice());
      }
    }
  }
}
