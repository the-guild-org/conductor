use std::sync::Arc;

use actix_web::{
  dev::Response,
  route,
  web::{self, Bytes},
  App, HttpRequest, HttpResponse, HttpServer, Responder, Scope,
};
use conductor_common::http::{ConductorHttpRequest, ConductorHttpResponse, HttpHeadersMap};
use conductor_config::{load_config, LoggerConfig, LoggerConfigFormat};
use conductor_engine::gateway::{ConductorGateway, ConductorGatewayRouteData};
use tracing::{debug, error, info};
use tracing_subscriber::{
  fmt::{self, format::FmtSpan, time::UtcTime},
  layer::SubscriberExt,
  registry, EnvFilter,
};

pub async fn run_services(config_file_path: &String) -> std::io::Result<()> {
  // Yassin: we don't have tracing::subscriber logging before nor inside `load_config()` anymore

  println!("Gateway process started");
  let config_object = load_config(config_file_path, |key| std::env::var(key).ok()).await;
  println!("Configuration loaded and parsed");

  // default logger configuration
  let default_env_filter = String::from("info");
  let logger_config = config_object.logger.clone().unwrap_or(LoggerConfig {
    env_filter: Some(default_env_filter.clone()),
    format: LoggerConfigFormat::Compact,
  });

  // initialize logging with `info` as a default filter, before we read the `logger` config from file
  let filter = EnvFilter::new(
    logger_config.env_filter.clone().unwrap_or(
      logger_config
        .env_filter
        .unwrap_or(default_env_filter.clone()),
    ),
  );

  match logger_config.format {
    LoggerConfigFormat::Json => {
      let fmt_layer = fmt::Layer::new()
        .with_timer(UtcTime::rfc_3339())
        .with_span_events(FmtSpan::CLOSE)
        .json();

      let subscriber = registry::Registry::default().with(filter).with(fmt_layer);
      tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up the json logger");
    }
    LoggerConfigFormat::Pretty => {
      let fmt_layer = fmt::Layer::new()
        .with_timer(UtcTime::rfc_3339())
        .with_span_events(FmtSpan::CLOSE)
        .pretty();

      let subscriber = registry::Registry::default().with(filter).with(fmt_layer);
      tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up the pretty logger");
    }
    LoggerConfigFormat::Compact => {
      let fmt_layer = fmt::Layer::new()
        .with_timer(UtcTime::rfc_3339())
        .with_span_events(FmtSpan::CLOSE)
        .compact();

      let subscriber = registry::Registry::default().with(filter).with(fmt_layer);
      tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set up the compact logger");
    }
  }

  debug!("building gateway from configuration...");
  match ConductorGateway::new(&config_object).await {
    Ok(gw) => {
      let gateway = Arc::new(gw);
      let http_server = HttpServer::new(move || {
        let mut router = App::new();

        for conductor_route in gateway.routes.iter() {
          let child_router = Scope::new(conductor_route.base_path.as_str())
            .app_data(web::Data::new(conductor_route.route_data.clone()))
            .route("{tail:.*}", web::route().to(handler))
            .route("", web::route().to(handler));

          router = router.service(child_router)
        }

        router.service(health_handler)
      });

      let server_config = config_object.server.clone().unwrap_or_default();
      let server_address = format!("{}:{}", server_config.host, server_config.port);
      debug!("server is trying to listen on {:?}", server_address);

      http_server
        .bind((server_config.host, server_config.port))?
        .run()
        .await
    }
    Err(e) => {
      error!("failed to initialize gateway: {:?}", e);
      panic!("Failed to initialize gateway: {:?}", e);
    }
  }
}

#[route("/_health", method = "GET", method = "HEAD")]
async fn health_handler() -> impl Responder {
  Response::ok()
}

#[tracing::instrument(level = "debug", skip(req, body))]
fn transform_req(req: HttpRequest, body: Bytes) -> ConductorHttpRequest {
  let mut headers_map = HttpHeadersMap::new();

  for (key, value) in req.headers().into_iter() {
    headers_map.insert(key, value.clone());
  }

  let conductor_request = ConductorHttpRequest {
    body,
    headers: headers_map,
    method: req.method().clone(),
    uri: req.uri().to_string(),
    query_string: req.query_string().to_string(),
  };

  conductor_request
}

#[tracing::instrument(level = "debug", skip(conductor_response))]
fn transform_res(conductor_response: ConductorHttpResponse) -> HttpResponse {
  let mut response = HttpResponse::build(conductor_response.status);

  for (key, value) in conductor_response.headers.iter() {
    response.insert_header((key, value));
  }

  response.body(conductor_response.body)
}

#[tracing::instrument(
  level = "debug",
  skip(req, body, route_data),
  name = "conductor_bin::handler"
)]
async fn handler(
  req: HttpRequest,
  body: Bytes,
  route_data: web::Data<Arc<ConductorGatewayRouteData>>,
) -> impl Responder {
  let conductor_request = transform_req(req, body);
  let conductor_response = ConductorGateway::execute(conductor_request, &route_data).await;

  transform_res(conductor_response)
}
