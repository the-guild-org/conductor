use std::collections::HashMap;

use async_graphql::http::GraphiQLSource;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::Router;
use hyper::service::{make_service_fn, Service};

use crate::{
    config::{ConductorConfig, SourceDefinition},
    source::source::SourceService,
    source::graphql_source::GraphQLSourceService, endpoint::endpoint::EndpointRuntime,
};
use axum::{
    extract::Extension,
    response::{self, IntoResponse},
    routing::get,
    Server,
    http::Request,
    body::Body,
};

async fn graphiql(req: Request<Body>) -> impl IntoResponse {
    response::Html(GraphiQLSource::build().endpoint(req.uri().path()).finish())
}

pub struct Gateway {
    configuration: ConductorConfig,
    sources: HashMap<String, Box<dyn SourceService>>,
    pub endpoints: HashMap<String, EndpointRuntime>
}

impl Gateway {
    pub fn new(configuration: ConductorConfig) -> Self {
        let clone = configuration.clone();

        let sources_map: HashMap<String, Box<dyn SourceService>> = configuration
            .sources
            .iter()
            .map::<(String, Box<dyn SourceService>), _>(|source_config| match source_config {
                SourceDefinition::GraphQL { id, config } => 
                    (id.clone(), Box::new(GraphQLSourceService::create(config.clone()))),
            })
            .collect();

        let endpoints_map: HashMap<String, EndpointRuntime> = configuration
            .endpoints
            .iter()
            .map::<(String, EndpointRuntime), _>(|endpoint_config| {
                (endpoint_config.path.clone(), EndpointRuntime::new(endpoint_config.clone()))
            })
            .collect();

        Self { configuration: clone, sources: sources_map, endpoints: endpoints_map }
    }

    pub fn get_routes(self: &Self) {
        // self.endpoints.iter().map(|(path, endpoint)| {
        //     async fn graphql_handler(req: GraphQLRequest) -> GraphQLResponse {
        //         todo!()
        //     }

        //     match endpoint.config.graphiql {
        //         true => get(graphiql).post(graphql_handler),
        //         false =>  get(graphiql),
        //     }
        // });
    }
}
