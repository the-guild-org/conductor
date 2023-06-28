use std::collections::HashMap;

use crate::{
    config::{ConductorConfig, SourceDefinition},
    endpoint::endpoint::EndpointRuntime,
    source::graphql_source::GraphQLSourceService,
    source::source::SourceService,
};

pub struct Gateway {
    pub configuration: ConductorConfig,
    pub sources: HashMap<String, Box<dyn SourceService>>,
    pub endpoints: HashMap<String, EndpointRuntime>,
}

impl Gateway {
    pub fn new(configuration: ConductorConfig) -> Self {
        let clone = configuration.clone();

        let sources_map: HashMap<String, Box<dyn SourceService>> = configuration
            .sources
            .iter()
            .map::<(String, Box<dyn SourceService>), _>(|source_config| match source_config {
                SourceDefinition::GraphQL { id, config } => (
                    id.clone(),
                    Box::new(GraphQLSourceService::create(config.clone())),
                ),
            })
            .collect();

        let endpoints_map: HashMap<String, EndpointRuntime> = configuration
            .endpoints
            .iter()
            .map::<(String, EndpointRuntime), _>(|endpoint_config| {
                (
                    endpoint_config.path.clone(),
                    EndpointRuntime::new(endpoint_config.clone()),
                )
            })
            .collect();

        Self {
            configuration: clone,
            sources: sources_map,
            endpoints: endpoints_map,
        }
    }
}
