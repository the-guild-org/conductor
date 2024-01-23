use std::str::FromStr;

use conductor_common::http::{
  ConductorHttpRequest, ConductorHttpResponse, HeaderName, HeaderValue, HttpHeadersMap, Method,
};
use conductor_config::parse_config_contents;
use conductor_engine::gateway::{ConductorGateway, GatewayError};
use std::panic;
use tracing_subscriber::prelude::*;
use worker::*;

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

#[tracing::instrument(level = "debug", skip(conductor_response), name = "transform_response")]
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

async fn run_flow(req: Request, env: Env) -> Result<Response> {
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

      match ConductorGateway::new(&conductor_config, &mut None).await {
        Ok(gw) => {
          let _ = tracing_subscriber::registry().with(logger).try_init();
          let url = req.url()?;

          match gw.match_route(&url) {
            Ok(route_data) => {
              let conductor_req = transform_req(&url, req).await?;
              let conductor_response = ConductorGateway::execute(conductor_req, route_data).await;

              transform_res(conductor_response)
            }
            Err(GatewayError::MissingEndpoint(_)) => {
              Response::error("failed to locate endpoint".to_string(), 404)
            }
            Err(e) => Response::error(e.to_string(), 500),
          }
        }
        Err(_) => Response::error("gateway is not ready".to_string(), 500),
      }
    }
    Err(e) => Response::error(e.to_string(), 500),
  }
}

#[event(start)]
fn start() {
  // This will make sure to capture runtime events from the WASM and print it to the log
  panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[event(fetch, respond_with_errors)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
  let result = run_flow(req, env).await;

  match result {
    Ok(response) => Ok(response),
    Err(e) => Response::error(e.to_string(), 500),
  }
}
