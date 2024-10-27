use conductor_tracing::otel_attrs::*;
use fastrace::Span;
use reqwest::{Request, Response, StatusCode};
use reqwest_middleware::ClientBuilder;
use reqwest_middleware::ClientWithMiddleware;
use reqwest_middleware::{Error, Middleware, Next, Result};

#[derive(Debug)]
pub struct MinitraceReqwestMiddleware;

#[inline]
fn get_span_status(request_status: StatusCode) -> Option<&'static str> {
  // adapted from: https://github.com/TrueLayer/reqwest-middleware/blob/main/reqwest-tracing/src/reqwest_otel_span_builder.rs#L149
  match request_status.as_u16() {
    // Span Status MUST be left unset if HTTP status code was in the 1xx, 2xx or 3xx ranges, unless there was
    // another error (e.g., network error receiving the response body; or 3xx codes with max redirects exceeded),
    // in which case status MUST be set to Error.
    100..=399 => None,
    // For HTTP status codes in the 4xx range span status MUST be left unset in case of SpanKind.SERVER and MUST be
    // set to Error in case of SpanKind.CLIENT.
    400..=499 => Some("ERROR"),
    // For HTTP status codes in the 5xx range, as well as any other code the client failed to interpret, span
    // status MUST be set to Error.
    _ => Some("ERROR"),
  }
}

impl MinitraceReqwestMiddleware {
  #[inline]
  pub fn response_properties(
    &self,
    res: &Result<Response>,
  ) -> impl IntoIterator<Item = (&'static str, String)> {
    let mut properties: Vec<(&'static str, String)> = vec![];

    match &res {
      Ok(response) => {
        let status_code = response.status().as_u16();

        let span_status = get_span_status(response.status());
        if let Some(span_status) = span_status {
          properties.push((OTEL_STATUS_CODE, span_status.to_string()));
          properties.push((ERROR_INDICATOR, "true".to_string()));
        }
        properties.push((HTTP_STATUS_CODE, status_code.to_string()));
      }
      Err(e) => {
        let error_message = e.to_string();
        let error_cause_chain = format!("{:?}", e);
        properties.push((OTEL_STATUS_CODE, "ERROR".to_string()));
        properties.push((ERROR_MESSAGE, error_message.to_string()));
        properties.push((ERROR_INDICATOR, "true".to_string()));
        properties.push((ERROR_CAUSE_CHAIN, error_cause_chain.to_string()));

        if let Error::Reqwest(e) = e {
          if let Some(status) = e.status() {
            properties.push((HTTP_STATUS_CODE, status.as_u16().to_string()));
          }
        }
      }
    };

    properties
  }

  #[inline]
  pub fn request_properties(
    &self,
    req: &Request,
  ) -> (String, impl IntoIterator<Item = (&'static str, String)>) {
    let method = req.method();
    let url = req.url();
    let scheme = url.scheme();
    let host = url.host_str().unwrap_or("");
    let host_port = url.port().unwrap_or(0) as i64;
    let otel_name = format!("{} {}", method, url.path());

    (
      otel_name,
      vec![
        (HTTP_METHOD, method.to_string()),
        (HTTP_SCHEME, scheme.to_string()),
        (HTTP_HOST, host.to_string()),
        (HTTP_URL, url.to_string()),
        (NET_HOST_PORT, host_port.to_string()),
        (OTEL_KIND, "client".to_string()),
        (SPAN_KIND, "consumer".to_string()),
      ],
    )
  }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl Middleware for MinitraceReqwestMiddleware {
  async fn handle(
    &self,
    req: Request,
    extensions: &mut http::Extensions,
    next: Next<'_>,
  ) -> Result<Response> {
    let (span_name, properties) = self.request_properties(&req);
    let mut _span_guard = Span::enter_with_local_parent(span_name).with_properties(|| properties);

    let response = next.run(req, extensions).await;

    _span_guard = _span_guard.with_properties(|| self.response_properties(&response));

    response
  }
}

pub fn traced_reqwest(raw_client: reqwest::Client) -> TracedHttpClient {
  ClientBuilder::new(raw_client)
    .with(MinitraceReqwestMiddleware)
    .build()
}

pub type TracedHttpClient = ClientWithMiddleware;
