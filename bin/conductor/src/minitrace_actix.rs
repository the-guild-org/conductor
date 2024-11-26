use actix_web::{
  dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
  http::{header::USER_AGENT, StatusCode, Version},
  web, Error, ResponseError,
};
use conductor_engine::gateway::ConductorGatewayRouteData;
use conductor_tracing::{otel_attrs::*, trace_id::generate_trace_id};
use fastrace::{
  collector::{SpanContext, SpanId},
  Span,
};
use futures_util::future::LocalBoxFuture;
use std::{
  future::{ready, Ready},
  sync::Arc,
};
use ulid::Ulid;

pub struct MinitraceTransform;

impl MinitraceTransform {
  pub fn new() -> Self {
    MinitraceTransform
  }
}

impl<S, B> Transform<S, ServiceRequest> for MinitraceTransform
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type InitError = ();
  type Transform = MinitraceMiddleware<S>;
  type Future = Ready<Result<Self::Transform, Self::InitError>>;

  fn new_transform(&self, service: S) -> Self::Future {
    ready(Ok(MinitraceMiddleware { service }))
  }
}

#[inline]
fn build_request_root_span(req: &ServiceRequest) -> Span {
  let endpoint_data = req
    .app_data::<web::Data<Arc<ConductorGatewayRouteData>>>()
    .expect("endpoint data not found, failed to setup tracing");

  let span_name = format!("HTTP {} {}", req.method(), req.path());
  let mut properties: Vec<(&str, String)> = build_request_properties(req);
  properties.push((CONDUCTOR_ENDPOINT, endpoint_data.endpoint.clone()));

  let span_context = SpanContext::new(
    generate_trace_id(endpoint_data.tenant_id),
    SpanId::default(),
  );

  Span::root(span_name, span_context).with_properties(|| properties)
}

#[inline]
pub fn http_flavor(version: Version) -> String {
  match version {
    Version::HTTP_09 => "0.9".into(),
    Version::HTTP_10 => "1.0".into(),
    Version::HTTP_11 => "1.1".into(),
    Version::HTTP_2 => "2.0".into(),
    Version::HTTP_3 => "3.0".into(),
    other => format!("{other:?}"),
  }
}

#[inline]
pub fn http_scheme(scheme: &str) -> String {
  match scheme {
    "http" => "http".into(),
    "https" => "https".into(),
    other => other.to_string(),
  }
}

fn gen_request_id() -> String {
  Ulid::new().to_string()
}

#[inline]
fn build_request_properties(req: &ServiceRequest) -> Vec<(&'static str, String)> {
  let headers = req.headers();
  let user_agent = headers
    .get(USER_AGENT)
    .map(|h| h.to_str().unwrap_or(""))
    .unwrap_or("");
  let http_route: std::borrow::Cow<'static, str> = req
    .match_pattern()
    .map(Into::into)
    .unwrap_or_else(|| "default".into());
  let http_method = req.method().to_string();
  let connection_info = req.connection_info();
  let request_id = headers
    .get("x-request-id")
    .map(|v| v.to_str().unwrap().to_string())
    .unwrap_or_else(gen_request_id);

  vec![
    (HTTP_METHOD, http_method),
    (HTTP_ROUTE, http_route.into_owned()),
    (HTTP_FLAVOR, http_flavor(req.version())),
    (HTTP_SCHEME, http_scheme(connection_info.scheme())),
    (HTTP_HOST, connection_info.host().to_string()),
    (
      HTTP_CLIENT_IP,
      connection_info
        .realip_remote_addr()
        .unwrap_or("")
        .to_string(),
    ),
    (HTTP_USER_AGENT, user_agent.to_string()),
    (
      HTTP_TARGET,
      req
        .uri()
        .path_and_query()
        .map(|p| p.as_str())
        .unwrap_or("")
        .to_string(),
    ),
    (OTEL_KIND, "server".to_string()),
    (REQUEST_ID, request_id),
    // Specific to DataDog
    (SPAN_TYPE, "web".to_string()),
  ]
}

#[inline]
fn handle_error(
  status_code: StatusCode,
  response_error: &dyn ResponseError,
) -> Vec<(&'static str, String)> {
  let mut properties: Vec<(&'static str, String)> = vec![];

  // pre-formatting errors is a workaround for https://github.com/tokio-rs/tracing/issues/1565
  let display = format!("{response_error}");
  let debug = format!("{response_error:?}");
  properties.push((EXCEPTION_MESSAGE, display));
  properties.push((EXCEPTION_DETAILS, debug));
  properties.push((ERROR_INDICATOR, "true".to_string()));

  let code = status_code.as_u16().to_string();
  properties.push((HTTP_STATUS_CODE, code));

  if status_code.is_client_error() {
    properties.push((OTEL_STATUS_CODE, "OK".to_string()));
  } else {
    properties.push((OTEL_STATUS_CODE, "ERROR".to_string()));
  }

  properties
}

#[inline]
fn build_response_properties<B>(
  res: &Result<ServiceResponse<B>, actix_web::Error>,
) -> Vec<(&'static str, String)> {
  let mut properties: Vec<(&'static str, String)> = vec![];

  match res {
    Ok(response) => {
      if let Some(error) = response.response().error() {
        properties.append(&mut handle_error(
          response.status(),
          error.as_response_error(),
        ));
      } else {
        let code = response.response().status().as_u16().to_string();
        properties.push((HTTP_STATUS_CODE, code));
        properties.push((OTEL_STATUS_CODE, "OK".to_string()));
      }
    }
    Err(error) => {
      let response_error = error.as_response_error();
      properties.append(&mut handle_error(
        response_error.status_code(),
        error.as_response_error(),
      ));
    }
  };

  properties
}

pub struct MinitraceMiddleware<S> {
  service: S,
}

impl<S, B> Service<ServiceRequest> for MinitraceMiddleware<S>
where
  S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
  S::Future: 'static,
  B: 'static,
{
  type Response = ServiceResponse<B>;
  type Error = Error;
  type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

  forward_ready!(service);

  fn call(&self, req: ServiceRequest) -> Self::Future {
    let root_span = build_request_root_span(&req);
    let fut = self.service.call(req);

    Box::pin(async move {
      let _guard = root_span.set_local_parent();
      let res = fut.await;
      let res_properties = build_response_properties(&res);
      let _ = root_span.with_properties(|| res_properties);

      res
    })
  }
}
