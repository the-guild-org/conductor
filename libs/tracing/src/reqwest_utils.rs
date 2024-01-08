use reqwest::Request;
use reqwest::Response;
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_middleware::Result;
use reqwest_tracing::default_on_request_end;
use reqwest_tracing::default_span_name;
use reqwest_tracing::reqwest_otel_span;
use reqwest_tracing::ReqwestOtelSpanBackend;
use reqwest_tracing::TracingMiddleware;
use task_local_extensions::Extensions;
use tracing::Span;

pub type TracedHttpClient = ClientWithMiddleware;

struct ReqwestSpanBackend;

impl ReqwestOtelSpanBackend for ReqwestSpanBackend {
  fn on_request_start(req: &Request, ext: &mut Extensions) -> Span {
    let name = default_span_name(req, ext);

    reqwest_otel_span!(
      name = name,
      req,
      "span.type" = "http",
      "span.kind" = "consumer",
    )
  }

  fn on_request_end(span: &Span, outcome: &Result<Response>, _: &mut Extensions) {
    default_on_request_end(span, outcome)
  }
}

pub fn traced_reqwest(raw_client: reqwest::Client) -> TracedHttpClient {
  ClientBuilder::new(raw_client)
    .with(TracingMiddleware::<ReqwestSpanBackend>::new())
    .build()
}
