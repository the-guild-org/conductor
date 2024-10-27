use std::{collections::HashMap, net::SocketAddr};

use conductor_tracing::reporters::AsyncReporter;
use fastrace::collector::SpanRecord;
use rmp_serde::Serializer;
use serde::Serialize;

pub struct WasmDatadogReporter {
  agent_endpoint: SocketAddr,
  service_name: String,
  resource: String,
  trace_type: String,
}

#[derive(Serialize)]
struct DatadogSpan<'a> {
  name: &'a str,
  service: &'a str,
  #[serde(rename = "type")]
  trace_type: &'a str,
  resource: &'a str,
  start: i64,
  duration: i64,
  #[serde(skip_serializing_if = "Option::is_none")]
  meta: Option<HashMap<&'a str, &'a str>>,
  error_code: i32,
  span_id: u64,
  trace_id: u64,
  parent_id: u64,
}

impl WasmDatadogReporter {
  pub fn new(
    agent_endpoint: &SocketAddr,
    service_name: impl Into<String>,
    resource: impl Into<String>,
    trace_type: impl Into<String>,
  ) -> Self {
    Self {
      agent_endpoint: agent_endpoint.clone(),
      service_name: service_name.into(),
      resource: resource.into(),
      trace_type: trace_type.into(),
    }
  }

  fn convert<'a>(&'a self, spans: &'a [SpanRecord]) -> Vec<DatadogSpan<'a>> {
    spans
      .iter()
      .map(move |s| DatadogSpan {
        name: &s.name,
        service: &self.service_name,
        trace_type: &self.trace_type,
        resource: &self.resource,
        start: s.begin_time_unix_ns as i64,
        duration: s.duration_ns as i64,
        meta: if s.properties.is_empty() {
          None
        } else {
          Some(
            s.properties
              .iter()
              .map(|(k, v)| (k.as_ref(), v.as_ref()))
              .collect(),
          )
        },
        error_code: 0,
        span_id: s.span_id.0,
        trace_id: s.trace_id.0 as u64,
        parent_id: s.parent_id.0,
      })
      .collect()
  }

  fn serialize(&self, spans: Vec<DatadogSpan>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buf = vec![0b10010001];
    spans.serialize(&mut Serializer::new(&mut buf).with_struct_map())?;
    Ok(buf)
  }

  async fn try_report(&self, spans: &[SpanRecord]) -> Result<(), Box<dyn std::error::Error>> {
    let datadog_spans = self.convert(spans);
    let bytes = self.serialize(datadog_spans)?;
    let client = reqwest::Client::new();
    let response = client
      .post(format!("http://{}/v0.4/traces", self.agent_endpoint))
      .header("Datadog-Meta-Tracer-Version", "v1.27.0")
      .header("Content-Type", "application/msgpack")
      .body(bytes)
      .send()
      .await?;

    tracing::debug!("datadog report done with status: {:?}", response.status());

    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl AsyncReporter for WasmDatadogReporter {
  async fn flush(&mut self, spans: &[SpanRecord]) {
    if spans.is_empty() {
      return;
    }

    if let Err(err) = self.try_report(spans).await {
      tracing::error!("report to datadog failed: {}", err);
    } else {
      tracing::debug!("flushed {} traces to datadog agent", spans.len());
    }
  }
}
