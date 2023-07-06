use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    config::EndpointDefinition,
    source::{
        base_source::{SourceError, SourceRequest, SourceService},
        graphql_source::GraphQLSourceService,
    },
    PluginsManager,
};
use hyper::{Body, Request, Response};

pub type EndpointResponse = hyper::Response<Body>;

pub struct PluginError {
    // still the implementation has to be discussed
}

pub trait OnRequestPlugin {
    fn on_request(&self, req: &mut Request<Body>) -> Result<(), PluginError>;
}

pub trait OnResponsePlugin {
    fn on_response(&self, res: &mut Response<Body>) -> Result<(), PluginError>;
}

pub struct EndpointRuntime {
    pub config: EndpointDefinition,
    // pub sources: Arc<HashMap<String, Arc<Mutex<GraphQLSourceService>>>>,
    pub upstream_service: Arc<Mutex<dyn SourceService + Send>>,

    pub plugins_manager: PluginsManager,
}

impl EndpointRuntime {
    pub fn new(
        config: EndpointDefinition,
        sources: Arc<HashMap<String, GraphQLSourceService>>,
        plugins_manager: PluginsManager,
    ) -> Self {
        let upstream_service = match sources.get(&config.from) {
            Some(e) => e.to_owned(),
            None => {
                panic!("source {} not found!", config.from);
            }
        };

        Self {
            config,
            upstream_service: Arc::new(Mutex::new(upstream_service)),
            plugins_manager,
        }
    }

    pub async fn call(&self, body: String) -> Result<EndpointResponse, SourceError> {
        let source_request = SourceRequest::new(body).await;

        // Run on_request for all plugins
        // for plugin in &self.plugins_manager.execute_on_request() {
        //     plugin.exec(&mut req)?;
        // }

        let future = self
            .upstream_service
            .lock()
            .expect("upstream service lock coudln't be acquired")
            .call(source_request);

        // // Run on_response for all plugins
        // for plugin in &self.plugins_manager.execute_post_response() {
        //     plugin.exec(&mut result)?;
        // }

        future.await
    }
}
