use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use minitrace::collector::{Reporter, SpanRecord};

use crate::reporters::TracingReporter;

pub type RouterFn = fn(&SpanRecord) -> Option<u32>;

pub struct RoutedReporter {
  reporters: HashMap<u32, Arc<Mutex<TracingReporter>>>,
  router_fn: RouterFn,
}

impl RoutedReporter {
  pub fn new(router_fn: RouterFn) -> Self {
    Self {
      reporters: HashMap::new(),
      router_fn,
    }
  }

  pub fn with_reporter(mut self, tenant_id: u32, reporter: Arc<Mutex<TracingReporter>>) -> Self {
    self.reporters.insert(tenant_id, reporter);

    self
  }

  pub fn make_report(&mut self, spans: &[SpanRecord]) {
    let mut chunks: HashMap<u32, Vec<SpanRecord>> = HashMap::new();

    for span in spans {
      if let Some(key) = (self.router_fn)(span) {
        let chunk = chunks.entry(key).or_default();
        chunk.push(span.clone());
      } else {
        tracing::warn!("no key for span: {:?}, dropping span", span);
      }
    }

    for (key, chunk) in chunks {
      if let Some(reporter) = self.reporters.get_mut(&key) {
        let mut r = reporter.lock().unwrap();
        r.report(&chunk);
      }
    }
  }
}

impl Reporter for RoutedReporter {
  #[cfg(target_arch = "wasm32")]
  fn report(&mut self, spans: &[SpanRecord]) {
    self.make_report(spans);
  }

  #[cfg(not(target_arch = "wasm32"))]
  fn report(&mut self, spans: &[SpanRecord]) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async move {
      self.make_report(spans);
    });
  }
}

#[cfg(feature = "test_utils")]
pub mod test_utils {
  use std::sync::{Arc, Mutex};

  use minitrace::collector::{Reporter, SpanRecord};

  pub struct TestReporter {
    captured_spans: Arc<Mutex<Vec<SpanRecord>>>,
  }

  impl TestReporter {
    pub fn new() -> (Arc<Mutex<Vec<SpanRecord>>>, Self) {
      let spans: Arc<Mutex<Vec<SpanRecord>>> = Arc::new(Mutex::new(vec![]));

      (
        spans.clone(),
        Self {
          captured_spans: spans,
        },
      )
    }
  }

  impl Reporter for TestReporter {
    fn report(&mut self, spans: &[SpanRecord]) {
      for span in spans.iter() {
        self.captured_spans.lock().unwrap().push(span.clone());
      }
    }
  }
}

// #[cfg(test)]
// mod test {
//   use super::*;

//   #[tokio::test]
//   async fn routed_reporter() {
//     let (spans0, reporter0) = test_utils::TestReporter::new();
//     let (spans1, reporter1) = test_utils::TestReporter::new();
//     let mut routed_reporter =
//       RoutedReporter::new(|span| Some(MinitraceManager::extract_tenant_id(span.trace_id)))
//         .with_reporter(0, Box::new(reporter0).into())
//         .with_reporter(1, Box::new(reporter1).into());

//     routed_reporter
//       .report(&vec![
//         // This one goes to tenant 2, it does not exists
//         SpanRecord {
//           trace_id: MinitraceManager::generate_trace_id(2),
//           ..Default::default()
//         },
//         // This one goes to tenant 0
//         SpanRecord {
//           trace_id: MinitraceManager::generate_trace_id(0),
//           ..Default::default()
//         },
//         // This one goes to tenant 0
//         SpanRecord {
//           trace_id: MinitraceManager::generate_trace_id(0),
//           ..Default::default()
//         },
//         // This one goes to tenant 1
//         SpanRecord {
//           trace_id: MinitraceManager::generate_trace_id(1),
//           ..Default::default()
//         },
//         // This one goes to tenant 2, it does not exists
//         SpanRecord {
//           trace_id: MinitraceManager::generate_trace_id(2),
//           ..Default::default()
//         },
//       ])
//       .await;

//     routed_reporter
//       .report(&vec![
//         // This one goes to tenant 1
//         SpanRecord {
//           trace_id: MinitraceManager::generate_trace_id(1),
//           ..Default::default()
//         },
//         // This one goes to tenant 1
//         SpanRecord {
//           trace_id: MinitraceManager::generate_trace_id(1),
//           ..Default::default()
//         },
//         // This one goes to tenant 2
//         SpanRecord {
//           trace_id: MinitraceManager::generate_trace_id(2),
//           ..Default::default()
//         },
//       ])
//       .await;

//     let spans0 = spans0.lock().unwrap();
//     let spans1 = spans1.lock().unwrap();

//     assert_eq!(spans0.len(), 2);
//     assert_eq!(spans1.len(), 3);
//   }
// }
