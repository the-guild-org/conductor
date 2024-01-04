mod http_tracing;
use std::{str::FromStr, sync::Arc};

use conductor_cache::cache_manager::CacheManager;
use conductor_common::http::{
  ConductorHttpRequest, ConductorHttpResponse, HeaderName, HeaderValue, HttpHeadersMap, Method,
};
use conductor_config::{parse_config_contents, LoggerConfig};
use conductor_engine::gateway::{ConductorGateway, GatewayError};
use conductor_tracing::minitrace_mgr::MinitraceManager;
use http_tracing::{build_request_root_span, build_response_properties};
use minitrace::{collector::Config, trace};
use std::panic;
use tracing::subscriber::set_global_default;
use tracing_subscriber::prelude::*;
use worker::*;

#[trace(name = "transform_request")]
async fn transform_req(url: &Url, mut req: Request) -> Result<ConductorHttpRequest> {
  let mut headers_map = HttpHeadersMap::new();

  for (k, v) in req.headers().entries() {
    if let (Ok(hn), Ok(hv)) = (HeaderName::from_str(&k), HeaderValue::from_str(&v)) {
      headers_map.insert(hn, hv);
    }
  }

  let body = req.bytes().await?;
  let uri = url.to_string();
  let query_string = url.query().unwrap_or_default().to_string();
  let method = Method::from_str(req.method().as_ref()).map_err(|e| e.to_string())?;

  Ok(ConductorHttpRequest {
    body: body.into(),
    uri,
    query_string,
    method,
    headers: headers_map,
  })
}

#[trace(name = "transform_response")]
fn transform_res(conductor_response: ConductorHttpResponse) -> Result<Response> {
  let mut response_headers = Headers::new();
  for (k, v) in conductor_response.headers.into_iter() {
    if let Some(ks) = k {
      if let Ok(vs) = v.to_str() {
        response_headers.append(ks.as_str(), vs)?
      }
    }
  }

  Response::from_bytes(conductor_response.body.into()).map(|r| {
    r.with_status(conductor_response.status.as_u16())
      .with_headers(response_headers)
  })
}

async fn run_flow(
  req: Request,
  env: Env,
  minitrace_mgr: &mut MinitraceManager,
) -> Result<Response> {
  let conductor_config_str = env.var("CONDUCTOR_CONFIG").map(|v| v.to_string());
  let get_env_value = |key: &str| env.var(key).map(|s| s.to_string()).ok();

  match conductor_config_str {
    Ok(conductor_config_str) => {
      let conductor_config = parse_config_contents(
        conductor_config_str,
        conductor_config::ConfigFormat::Yaml,
        get_env_value,
      );

      let logger_config = conductor_config.logger.clone().unwrap_or_default();
      let logger = conductor_logger::logger_layer::build_logger(
        &logger_config.format,
        &logger_config.filter,
        logger_config.print_performance_info,
      )
      .unwrap_or_else(|e| panic!("failed to build logger: {}", e));

      let cache_manager = Arc::new(CacheManager::new(
        conductor_config.cache_stores.clone().unwrap_or_default(),
      ));
      let result =
        match ConductorGateway::new(&conductor_config, minitrace_mgr, cache_manager).await {
          Ok(gw) => {
            let _guard =
              tracing::subscriber::set_default(tracing_subscriber::registry().with(logger));
            let root_reporter = minitrace_mgr.build_root_reporter();
            minitrace::set_reporter(root_reporter, Config::default());

            let url = req.url()?;

            match gw.match_route(&url) {
              Ok(route_data) => {
                let root_span =
                  build_request_root_span(route_data.tenant_id, &route_data.endpoint, &req);
                let _guard = root_span.set_local_parent();
                let conductor_req = transform_req(&url, req).await?;
                let conductor_response = ConductorGateway::execute(conductor_req, route_data).await;
                let http_response = transform_res(conductor_response);
                let res_properties = build_response_properties(&http_response);
                let _ = root_span.with_properties(|| res_properties);

                http_response
              }
              Err(GatewayError::MissingEndpoint(_)) => {
                Response::error("failed to locate endpoint".to_string(), 404)
              }
              Err(e) => Response::error(e.to_string(), 500),
            }
          }
          Err(_) => Response::error("gateway is not ready".to_string(), 500),
        };

      result
    }
    Err(e) => Response::error(format!("failed to read conductor config: {}", e), 500),
  }
}

#[event(start)]
fn start() {
  // This will make sure to capture runtime events from the WASM and print it to the log
  panic::set_hook(Box::new(console_error_panic_hook::hook));

  let default_logger_config = LoggerConfig::default();
  let global_logger = conductor_logger::logger_layer::build_logger(
    &default_logger_config.format,
    &default_logger_config.filter,
    default_logger_config.print_performance_info,
  )
  .expect("failed to build logger");
  set_global_default(tracing_subscriber::registry().with(global_logger))
    .expect("failed to set global default logger");
}

#[event(fetch, respond_with_errors)]
async fn main(req: Request, env: Env, context: Context) -> Result<Response> {
  let mut minitrace_mgr = MinitraceManager::default();
  let result = run_flow(req, env, &mut minitrace_mgr).await;

  match result {
    Ok(response) => {
      context.wait_until(async move {
        minitrace_mgr.shutdown().await;
      });

      Ok(response)
    }
    Err(e) => Response::error(e.to_string(), 500),
  }
}
