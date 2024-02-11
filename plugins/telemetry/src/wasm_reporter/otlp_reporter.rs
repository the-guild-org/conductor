use std::borrow::Cow;

use conductor_common::http::Bytes;
use conductor_tracing::reporters::AsyncReporter;
use http::{Request, Response};
use minitrace::collector::{EventRecord, SpanRecord};
use opentelemetry::{
  trace::{Event, SpanContext, SpanKind, Status, TraceFlags, TraceState},
  InstrumentationLibrary, Key, KeyValue, StringValue, Value,
};
use opentelemetry_http::{HttpClient, HttpError};
use opentelemetry_sdk::{
  export::trace::{SpanData, SpanExporter},
  trace::EvictedQueue,
  Resource,
};
use web_time::web::SystemTimeExt;
use web_time::{Duration, UNIX_EPOCH};

#[derive(Debug)]
pub struct WasmTracingHttpClient;

#[async_trait::async_trait]
impl HttpClient for WasmTracingHttpClient {
  async fn send(&self, request: Request<Vec<u8>>) -> Result<Response<Bytes>, HttpError> {
    wasm_polyfills::call_async(async move {
      let request = request.try_into()?;
      let client = wasm_polyfills::create_http_client().build().unwrap();

      let mut response = client.execute(request).await?.error_for_status()?;
      let headers = std::mem::take(response.headers_mut());
      let mut http_response = Response::builder()
        .status(response.status())
        .body(response.bytes().await?)?;

      *http_response.headers_mut() = headers;

      Ok(http_response)
    })
    .await
  }
}

pub struct WasmOtlpReporter {
  span_kind: SpanKind,
  resource: Cow<'static, Resource>,
  instrumentation_lib: InstrumentationLibrary,
  opentelemetry_exporter: Box<dyn SpanExporter>,
}

impl WasmOtlpReporter {
  pub fn new(
    exporter: impl SpanExporter + 'static,
    span_kind: SpanKind,
    resource: Cow<'static, Resource>,
    instrumentation_lib: InstrumentationLibrary,
  ) -> Self {
    Self {
      opentelemetry_exporter: Box::new(exporter),
      span_kind,
      resource,
      instrumentation_lib,
    }
  }

  fn convert(&self, spans: &[SpanRecord]) -> Vec<SpanData> {
    spans
      .iter()
      .map(move |span| SpanData {
        span_context: SpanContext::new(
          span.trace_id.0.into(),
          span.span_id.0.into(),
          TraceFlags::default(),
          false,
          TraceState::default(),
        ),
        dropped_attributes_count: 0,
        parent_span_id: span.parent_id.0.into(),
        name: span.name.clone(),
        start_time: (UNIX_EPOCH + Duration::from_nanos(span.begin_time_unix_ns)).to_std(),
        end_time: (UNIX_EPOCH + Duration::from_nanos(span.begin_time_unix_ns + span.duration_ns))
          .to_std(),
        attributes: Self::convert_properties(&span.properties),
        events: Self::convert_events(&span.events),
        links: EvictedQueue::new(0),
        status: Status::default(),
        span_kind: self.span_kind.clone(),
        resource: self.resource.clone(),
        instrumentation_lib: self.instrumentation_lib.clone(),
      })
      .collect()
  }

  fn convert_properties(properties: &[(Cow<'static, str>, Cow<'static, str>)]) -> Vec<KeyValue> {
    let mut map = Vec::new();
    for (k, v) in properties {
      map.push(KeyValue::new(
        cow_to_otel_key(k.clone()),
        cow_to_otel_value(v.clone()),
      ));
    }
    map
  }

  fn convert_events(events: &[EventRecord]) -> EvictedQueue<Event> {
    let mut queue = EvictedQueue::new(u32::MAX);
    queue.extend(events.iter().map(|event| {
      Event::new(
        event.name.clone(),
        (UNIX_EPOCH + Duration::from_nanos(event.timestamp_unix_ns)).to_std(),
        event
          .properties
          .iter()
          .map(|(k, v)| KeyValue::new(cow_to_otel_key(k.clone()), cow_to_otel_value(v.clone())))
          .collect(),
        0,
      )
    }));
    queue
  }

  async fn try_report(&mut self, spans: &[SpanRecord]) -> Result<(), Box<dyn std::error::Error>> {
    let opentelemetry_spans = self.convert(spans);
    self
      .opentelemetry_exporter
      .export(opentelemetry_spans)
      .await?;

    Ok(())
  }
}

#[async_trait::async_trait(?Send)]
impl AsyncReporter for WasmOtlpReporter {
  async fn flush(&mut self, spans: &[SpanRecord]) {
    if spans.is_empty() {
      return;
    }

    if let Err(err) = self.try_report(spans).await {
      tracing::error!("report to otlp failed: {:?}", err);
    } else {
      tracing::debug!("flushed {} traces to otlp", spans.len());
    }
  }
}

fn cow_to_otel_key(cow: Cow<'static, str>) -> Key {
  match cow {
    Cow::Borrowed(s) => Key::from_static_str(s),
    Cow::Owned(s) => Key::from(s),
  }
}

fn cow_to_otel_value(cow: Cow<'static, str>) -> Value {
  match cow {
    Cow::Borrowed(s) => Value::String(StringValue::from(s)),
    Cow::Owned(s) => Value::String(StringValue::from(s)),
  }
}
