use std::str::FromStr;

use conductor_common::http::{
    ConductorHttpRequest, HeaderName, HeaderValue, HttpHeadersMap, Method,
};
use conductor_config::from_yaml;
use conductor_engine::gateway::ConductorGateway;
use std::panic;
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::prelude::*;
use tracing_web::MakeConsoleWriter;
use worker::*;

async fn run_flow(mut req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let conductor_config_str = env.var("CONDUCTOR_CONFIG").map(|v| v.to_string());

    match conductor_config_str {
        Ok(conductor_config_str) => match from_yaml(&conductor_config_str) {
            Ok(conductor_config) => {
                let gw = ConductorGateway::lazy(conductor_config);

                if let Some(route_data) = gw.match_route(&req.url().unwrap()) {
                    console_log!("Route found: {:?}", route_data);
                    let mut headers_map = HttpHeadersMap::new();

                    for (k, v) in req.headers().entries() {
                        headers_map.insert(
                            HeaderName::from_str(&k).unwrap(),
                            HeaderValue::from_str(&v).unwrap(),
                        );
                    }

                    let body = req.bytes().await.unwrap().into();
                    let uri = req.url().unwrap().to_string();
                    let query_string = req.url().unwrap().query().unwrap_or_default().to_string();
                    let method = Method::from_str(req.method().as_ref()).unwrap();

                    let conductor_req = ConductorHttpRequest {
                        body,
                        uri,
                        query_string,
                        method,
                        headers: headers_map,
                    };

                    let conductor_response = gw.execute(conductor_req, &route_data).await;

                    let mut response_headers = Headers::new();
                    for (k, v) in conductor_response.headers.into_iter() {
                        response_headers
                            .append(k.unwrap().as_str(), v.to_str().unwrap())
                            .unwrap();
                    }

                    Response::from_bytes(conductor_response.body.into()).map(|r| {
                        r.with_status(conductor_response.status.as_u16())
                            .with_headers(response_headers)
                    })
                } else {
                    Response::error("No route found", 404)
                }
            }
            Err(e) => Response::error(e.to_string(), 500),
        },
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
