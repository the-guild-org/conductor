use std::sync::Arc;

use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, Error};
use conductor_engine::gateway::ConductorGatewayRouteData;
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};

pub struct ActixRootSpanBuilder;

impl RootSpanBuilder for ActixRootSpanBuilder {
  fn on_request_start(request: &ServiceRequest) -> Span {
    let endpoint_data = request
      .app_data::<web::Data<Arc<ConductorGatewayRouteData>>>()
      .map(|v| &v.endpoint);

    match endpoint_data {
      Some(endpoint) => {
        let span_name = format!("{} {}", request.method(), request.path());
        tracing_actix_web::root_span!(
          request,
          endpoint = endpoint,
          "otel.name" = span_name,
          "span.type" = "web",
        )
      }
      None => tracing_actix_web::root_span!(request),
    }
  }

  fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
    DefaultRootSpanBuilder::on_request_end(span, outcome);
  }
}
