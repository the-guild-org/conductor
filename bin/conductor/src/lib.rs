use std::{collections::HashMap, env::vars};

use actix_web::{
    body::MessageBody,
    dev::{Response, ServiceFactory, ServiceRequest, ServiceResponse},
    route,
    web::{self, Bytes},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Scope,
};
use conductor_common::http::{ConductorHttpRequest, HttpHeadersMap};
use conductor_config::{interpolate::ConductorEnvVars, load_config, ConductorConfig};
use conductor_engine::gateway::{ConductorGateway, ConductorGatewayRouteData};
use tracing::debug;
use tracing_subscriber::fmt::format::FmtSpan;

struct EnvVarsFetcher {
    vars_map: HashMap<String, String>,
}

impl EnvVarsFetcher {
    pub fn new() -> Self {
        Self {
            vars_map: vars().collect::<HashMap<String, String>>(),
        }
    }
}

impl ConductorEnvVars for EnvVarsFetcher {
    fn get_var(&self, key: &str) -> Option<String> {
        self.vars_map.get(key).cloned()
    }
}

pub async fn run_services(config_file_path: &String) -> std::io::Result<()> {
    println!("gateway process started");
    println!("loading configuration from {}", config_file_path);
    let config_object = load_config(config_file_path, EnvVarsFetcher::new()).await;
    println!("configuration loaded and parsed");

    let logger_config = config_object.logger.clone();
    tracing_subscriber::fmt()
        .with_max_level(logger_config.unwrap_or_default().level.into_level())
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let server_config = config_object.server.clone().unwrap_or_default();
    let server_address = format!("{}:{}", server_config.host, server_config.port);
    debug!("server is trying to listen on {:?}", server_address);

    HttpServer::new(move || create_router_from_config(config_object.clone()))
        .bind((server_config.host, server_config.port))?
        .run()
        .await
}

fn create_router_from_config(
    config_object: ConductorConfig,
) -> App<
    impl ServiceFactory<
        ServiceRequest,
        Response = ServiceResponse<impl MessageBody>,
        Config = (),
        InitError = (),
        Error = Error,
    >,
> {
    let root_router = App::new();

    let (gateway, root_router) = ConductorGateway::new_with_external_router(
        config_object,
        root_router,
        &mut |route_data, app, path| {
            let child_router = Scope::new(path.as_str())
                .app_data(web::Data::new(route_data))
                .route("{tail:.*}", web::route().to(handler))
                .route("", web::route().to(handler));

            app.service(child_router)
        },
    );

    root_router
        .app_data(web::Data::new(gateway))
        .service(health_handler)
}

#[route("/_health", method = "GET", method = "HEAD")]
async fn health_handler() -> impl Responder {
    println!("health check");
    Response::ok()
}

async fn handler(
    req: HttpRequest,
    body: Bytes,
    route_data: web::Data<ConductorGatewayRouteData>,
    gw: web::Data<ConductorGateway>,
) -> impl Responder {
    let mut headers_map = HttpHeadersMap::new();
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
