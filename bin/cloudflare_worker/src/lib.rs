use std::str::FromStr;

use conductor_common::http::{
  ConductorHttpRequest, HeaderName, HeaderValue, HttpHeadersMap, Method,
};
use conductor_config::parse_config_contents;
use conductor_engine::gateway::{ConductorGateway, GatewayError};
use std::panic;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_web::MakeConsoleWriter;
use worker::*;
async fn run_flow(mut req: Request, env: Env, _ctx: Context) -> Result<Response> {
  let conductor_config_str = env.var("CONDUCTOR_CONFIG").map(|v| v.to_string());
  let get_env_value = |key: &str| env.var(key).map(|s| s.to_string()).ok();

  match conductor_config_str {
    Ok(conductor_config_str) => {
      let conductor_config = parse_config_contents(
        conductor_config_str,
        conductor_config::ConfigFormat::Yaml,
        get_env_value,
      );

      match ConductorGateway::new(&conductor_config).await {
        Ok(gw) => {
          let url = match req.url() {
            Ok(url) => url,
            Err(e) => return Response::error(e.to_string(), 500),
          };

          match gw.match_route(&url) {
            Ok(route_data) => {
              let mut headers_map = HttpHeadersMap::new();

              for (k, v) in req.headers().entries() {
                if let (Ok(hn), Ok(hv)) = (HeaderName::from_str(&k), HeaderValue::from_str(&v)) {
                  headers_map.insert(hn, hv);
                }
              }

              let body = match req.bytes().await {
                Ok(b) => b.into(),
                Err(e) => return Response::error(e.to_string(), 500),
              };
              let uri = url.to_string();
              let query_string = url.query().unwrap_or_default().to_string();
              let method = match Method::from_str(req.method().as_ref()) {
                Ok(m) => m,
                Err(e) => return Response::error(e.to_string(), 500),
              };

              let conductor_req = ConductorHttpRequest {
                body,
                uri,
                query_string,
                method,
                headers: headers_map,
              };

              let conductor_response = ConductorGateway::execute(conductor_req, route_data).await;

              let mut response_headers = Headers::new();
              for (k, v) in conductor_response.headers.into_iter() {
                if let Some(ks) = k.map(|k| k) {
                  if let Ok(vs) = v.to_str() {
                    response_headers
                      .append(ks.as_str(), vs)
                      .map_err(|e| e.to_string())?;
                  }
                }
              }

              Response::from_bytes(conductor_response.body.into()).map(|r| {
                r.with_status(conductor_response.status.as_u16())
                  .with_headers(response_headers)
              })
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

  // This will make sure to capture the logs from the WASM and print it to the log
  let fmt_layer = tracing_subscriber::fmt::layer()
    .json()
    .with_ansi(false)
    .with_timer(UtcTime::rfc_3339()) // std::time is not available in wasm env
    .with_writer(MakeConsoleWriter);
  tracing_subscriber::registry().with(fmt_layer).init();
}

#[event(fetch, respond_with_errors)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
  match run_flow(req, env, ctx).await {
    Ok(response) => Ok(response),
    Err(e) => Response::error(e.to_string(), 500),
  }
}
