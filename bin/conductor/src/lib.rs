mod minitrace_actix;

use std::sync::Arc;

use actix_web::{
  dev::Response,
  middleware::Compat,
  route,
  web::{self, Bytes},
  App, HttpRequest, HttpResponse, HttpServer, Responder, Scope,
};
use conductor_common::http::{ConductorHttpRequest, ConductorHttpResponse, HttpHeadersMap};

use crate::minitrace_actix::MinitraceTransform;

use conductor_config::load_config;
use conductor_engine::gateway::{ConductorGateway, ConductorGatewayRouteData};
use conductor_tracing::fastrace_mgr::FastraceManager;
use fastrace::{collector::Config, trace};
use tracing::{debug, error};
use tracing_subscriber::{layer::SubscriberExt, registry};

use actix_web::http::{
  header::HeaderName as ActixHeaderName, header::HeaderValue as ActixHeaderValue,
  Method as ActixMethod, StatusCode as ActixStatusCode,
};
use conductor_common::http::{
  HeaderName as ConductorHeaderName, HeaderValue as ConductorHeaderValue,
  Method as ConductorMethod, StatusCode as ConductorStatusCode,
};

fn convert_header_name(header: &ActixHeaderName) -> ConductorHeaderName {
  ConductorHeaderName::try_from(header.as_str())
    .unwrap_or_else(|_| panic!("Invalid header name: {}", header))
}

fn convert_header_value(value: &ActixHeaderValue) -> ConductorHeaderValue {
  ConductorHeaderValue::from_bytes(value.as_bytes()).expect("Invalid header value bytes")
}

fn convert_method(method: &ActixMethod) -> ConductorMethod {
  ConductorMethod::from_bytes(method.as_str().as_bytes()).expect("Invalid HTTP method")
}

fn convert_status_code(status: ConductorStatusCode) -> ActixStatusCode {
  ActixStatusCode::from_u16(status.as_u16()).unwrap_or(ActixStatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn run_services(config_file_path: &String) -> std::io::Result<()> {
  let config = load_config(config_file_path, |key| std::env::var(key).ok()).await;
  let logger_config = config.logger.clone().unwrap_or_default();
  let logger = conductor_logger::logger_layer::build_logger(
    &logger_config.format,
    &logger_config.filter,
    logger_config.print_performance_info,
  )
  .unwrap_or_else(|e| panic!("failed to build logger: {}", e));
  let mut tracing_manager = FastraceManager::default();

  match ConductorGateway::new(&config, &mut tracing_manager).await {
    Ok(gw) => {
      let subscriber = registry::Registry::default().with(logger);
      // @expected: we need to exit the process, if the logger can't be correctly set.
      let _guard = tracing::subscriber::set_default(subscriber);
      let tracing_reporter = tracing_manager.build_root_reporter();
      fastrace::set_reporter(tracing_reporter, Config::default());

      let gateway = Arc::new(gw);
      let http_server = HttpServer::new(move || {
        let mut router = App::new();

        for conductor_route in gateway.routes.iter() {
          let child_router = Scope::new(conductor_route.base_path.as_str())
            .wrap(Compat::new(MinitraceTransform::new()))
            .app_data(web::Data::new(conductor_route.route_data.clone()))
            .service(Scope::new("").default_service(
              web::route().to(handler), // handle all requests with this handler
            ));

          router = router.service(child_router)
        }

        router.service(health_handler)
      });

      let server_config = config.server.clone().unwrap_or_default();

      let server_address = format!("{}:{}", server_config.host, server_config.port);
      debug!("server is trying to listen on {:?}", server_address);

      let server_instance = http_server
        .bind((server_config.host, server_config.port))?
        .run()
        .await;

      tracing_manager.shutdown().await;

      server_instance
    }
    Err(e) => {
      error!("failed to initialize gateway: {:?}", e);
      // @expected: we need to exit the process, if the provided configuration file is incorrect.
      panic!("Failed to initialize gateway: {:?}", e);
    }
  }
}

#[route("/_health", method = "GET", method = "HEAD")]
async fn health_handler() -> impl Responder {
  Response::ok()
}

#[trace(name = "transform_request")]
fn transform_req(req: HttpRequest, body: Bytes) -> ConductorHttpRequest {
  let mut headers_map = HttpHeadersMap::new();

  for (key, value) in req.headers().iter() {
    headers_map.insert(convert_header_name(key), convert_header_value(value));
  }

  ConductorHttpRequest {
    body,
    headers: headers_map,
    method: convert_method(req.method()),
    uri: req.uri().to_string(),
    query_string: req.query_string().to_string(),
  }
}

#[trace(name = "transform_response")]
fn transform_res(conductor_response: ConductorHttpResponse) -> HttpResponse {
  let status = convert_status_code(conductor_response.status);
  let mut response = HttpResponse::build(status);

  for (key, value) in conductor_response.headers.iter() {
    let actix_key = ActixHeaderName::try_from(key.as_str()).expect("Invalid header name");
    let actix_value =
      ActixHeaderValue::from_str(value.to_str().unwrap()).expect("Invalid header value");

    response.insert_header((actix_key, actix_value));
  }

  response.body(conductor_response.body)
}

async fn handler(
  req: HttpRequest,
  body: Bytes,
  route_data: web::Data<Arc<ConductorGatewayRouteData>>,
) -> impl Responder {
  let conductor_request = transform_req(req, body);
  let conductor_response: ConductorHttpResponse =
    ConductorGateway::execute(conductor_request, &route_data).await;

  transform_res(conductor_response)
}
