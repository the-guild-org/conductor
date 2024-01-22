use std::collections::HashMap;

use minitrace::collector::{Reporter, SpanRecord};

pub struct MinitraceManager {
  reporters: HashMap<String, Box<dyn Reporter>>,
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

  pub fn add_reporter(&mut self, key: String, reporter: Box<dyn Reporter>) {
    self.reporters.insert(key, reporter);
  }

  pub fn build_reporter(self) -> impl Reporter {
    let mut routed_reporter =
      RoutedReporter::new(|span| span.metadata::<String>().map(|s| s.as_str()));

    for (key, reporter) in self.reporters {
      routed_reporter = routed_reporter.with_reporter(key.as_str(), reporter);
    }

    routed_reporter
  }
}

type RouterFn = fn(&SpanRecord) -> Option<&str>;

struct RoutedReporter {
  reporters: HashMap<String, Box<dyn Reporter>>,
  router_fn: RouterFn,
}

impl RoutedReporter {
  pub fn new(router_fn: RouterFn) -> Self {
    Self {
      reporters: HashMap::new(),
      router_fn,
    }
  }

  pub fn with_reporter(mut self, key: &str, reporter: Box<dyn Reporter>) -> Self {
    self.reporters.insert(key.to_string(), reporter);

    self
  }
}

impl Reporter for RoutedReporter {
  fn report(&mut self, spans: &[SpanRecord]) {
    let mut chunks: HashMap<&str, Vec<SpanRecord>> = HashMap::new();
    for span in spans {
      if let Some(key) = (self.router_fn)(span) {
        let chunk = chunks.entry(key).or_insert_with(Vec::new);
        chunk.push(span.clone());
      } else {
        tracing::warn!("no key for span: {:?}, dropping span", span);
      }
    }

    for (key, chunk) in chunks {
      if let Some(reporter) = self.reporters.get_mut(key) {
        reporter.report(chunk.as_slice());
      }
    }
  }
}
