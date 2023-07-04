use std::{collections::HashMap, sync::Arc};

use crate::{
    config::{ConductorConfig, SourceDefinition},
    endpoint::endpoint_runtime::EndpointRuntime,
    source::graphql_source::GraphQLSourceService,
};

pub struct Gateway {
    pub sources: Arc<HashMap<String, GraphQLSourceService>>,
    pub endpoints: HashMap<String, EndpointRuntime>,
}

impl Gateway {
    pub fn new(configuration: ConductorConfig) -> Self {
        let sources_map: HashMap<String, GraphQLSourceService> = configuration
            .sources
            .iter()
            .map::<(String, GraphQLSourceService), _>(|source_config| match source_config {
                SourceDefinition::GraphQL { id, config } => {
                    (id.clone(), GraphQLSourceService::create(config.clone()))
                }
            })
            .collect();

        let sources_map = Arc::new(sources_map);

        let endpoints_map: HashMap<String, EndpointRuntime> = configuration
            .endpoints
            .iter()
            .map::<(String, EndpointRuntime), _>(|endpoint_config| {
                (
                    endpoint_config.path.clone(),
                    EndpointRuntime::new(endpoint_config.clone(), sources_map.clone()),
                )
            })
            .collect();

        Self {
            sources: sources_map,
            endpoints: endpoints_map,
        }
    }
}
