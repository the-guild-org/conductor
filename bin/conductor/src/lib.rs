use std::sync::Arc;

use actix_web::{
  dev::Response,
  route,
  web::{self, Bytes},
  App, HttpRequest, HttpResponse, HttpServer, Responder, Scope,
};
use conductor_common::http::{ConductorHttpRequest, ConductorHttpResponse, HttpHeadersMap};
use conductor_config::load_config;
use conductor_engine::gateway::{ConductorGateway, ConductorGatewayRouteData};
use tracing::{debug, error, info};
use tracing_subscriber::{
  fmt::{self, format::FmtSpan, time::UtcTime},
  layer::SubscriberExt,
  registry, reload, EnvFilter,
};

pub async fn run_services(config_file_path: &String) -> std::io::Result<()> {
  // Initialize logging with `info` before we read the `logger` config from file
  let filter = EnvFilter::new("info");
  let (filter, reload_handle) = reload::Layer::new(filter);
  let subscriber = registry::Registry::default().with(filter).with(
    fmt::Layer::default()
      .with_timer(UtcTime::rfc_3339())
      .with_span_events(FmtSpan::CLOSE),
  );
  // Set the subscriber as the global default.
  tracing::subscriber::set_global_default(subscriber).expect("failed to set up the logger");

  info!("gateway process started");
  info!("loading configuration from {}", config_file_path);
  let config_object = load_config(config_file_path, |key| std::env::var(key).ok()).await;
  info!("configuration loaded and parsed");

  // If there's a logger config, modify the logging level to match the config
  if let Some(logger_config) = &config_object.logger {
    let new_level = logger_config.level.into_level().to_string();
    reload_handle
      .modify(|filter| {
        *filter = EnvFilter::new(new_level);
      })
      .expect("Failed to modify the log level");
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
  let headers_map = HttpHeadersMap::new();
  for (key, value) in req.headers().iter() {
    headers_map.insert(key, value.clone());
  }

  let conductor_request = ConductorHttpRequest {
    body,
    headers: headers_map,
    method: req.method().clone(),
    uri: req.uri().to_string(),
    query_string: req.query_string().to_string(),
  };

  let conductor_response = gw.execute(conductor_request, &route_data).await;

  let mut response = HttpResponse::build(conductor_response.status);

  for (key, value) in conductor_response.headers.iter() {
    response.insert_header((key, value));
  }

  response.body(conductor_response.body)
}
