use actix_web::{
    body::MessageBody,
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    web::{self, Bytes},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Scope,
};
use conductor_common::http::{ConductorHttpRequest, HttpHeadersMap};
use conductor_config::{load_config, ConductorConfig};
use conductor_engine::gateway::{ConductorGateway, ConductorGatewayRouteData};
use tracing::debug;

pub async fn run_services(config_file_path: &String) -> std::io::Result<()> {
    println!("gateway process started");
    println!("loading configuration from {}", config_file_path);
    let config_object = load_config(config_file_path).await;
    println!("configuration loaded");

    tracing_subscriber::fmt()
        .with_max_level(config_object.logger.level.into_level())
        .init();

    let server_config = config_object.server.clone();
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
                .route("/.*", web::route().to(handler));

            app.service(child_router)
        },
    );

    root_router.app_data(web::Data::new(gateway))
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
