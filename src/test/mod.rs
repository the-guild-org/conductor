#[cfg(test)]
pub mod utils {
    use crate::{
        config::{
            ConductorConfig, EndpointDefinition, GraphQLSourceConfig, LoggerConfig, ServerConfig,
        },
        create_router_from_config,
    };
    use axum_test::TestServer;
    use httpmock::prelude::*;
    use serde_json::json;

    pub struct ConductorTest {
        config: ConductorConfig,
    }

    impl ConductorTest {
        pub fn empty() -> Self {
            ConductorTest {
                config: ConductorConfig {
                    server: ServerConfig {
                        host: "localhost".to_string(),
                        port: 3000,
                    },
                    logger: LoggerConfig {
                        level: crate::config::Level(tracing::Level::TRACE),
                    },
                    endpoints: vec![],
                    plugins: None,
                    sources: vec![],
                },
            }
        }

        pub fn mocked_source(mut self) -> Self {
            let server = MockServer::start();
            server.mock(|when, then| {
                when.method(POST).path("/graphql");
                then.status(200)
                    .header("content-type", "application/json")
                    .body(
                        json!({
                          "data": {
                            "user": {
                              "id": "1"
                            }
                          },
                        })
                        .to_string(),
                    );
            });

            let source_id = "s".to_string();
            self.config
                .sources
                .push(crate::config::SourceDefinition::GraphQL {
                    id: source_id,
                    config: GraphQLSourceConfig {
                        endpoint: server.url("/graphql"),
                    },
                });

            self
        }

        pub fn endpoint(mut self, endpoint: EndpointDefinition) -> Self {
            self.config.endpoints.push(endpoint);

            self
        }

        pub fn finish(self) -> TestServer {
            let router = create_router_from_config(self.config);
            TestServer::new(router).expect("failed to create test server")
        }
    }
}
