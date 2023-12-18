use conductor_common::{graphql::GraphQLResponse, http::StatusCode};
use conductor_config::plugins::DisableIntrospectionPluginConfig;
use tracing::{error, warn};
use vrl::{
    compiler::{Context, Program, TargetValue, TimeZone},
    value,
    value::Secrets,
};

use crate::request_execution_context::RequestExecutionContext;

use super::{
    core::Plugin,
    vrl::{utils::conductor_request_to_value, vrl_functions::vrl_fns},
};

pub struct DisableIntrospectionPlugin {
    condition: Option<Program>,
}

impl DisableIntrospectionPlugin {
    pub fn new(config: DisableIntrospectionPluginConfig) -> Self {
        match &config.condition {
            Some(condition) => match vrl::compiler::compile(condition.contents(), &vrl_fns()) {
                Err(err) => {
                    error!("vrl compiler error: {:?}", err);
                    panic!("failed to compile vrl program for disable_introspection plugin");
                }
                Ok(result) => {
                    if result.warnings.len() > 0 {
                        warn!(
                            "vrl compiler warning for disable_introspection plugin: {:?}",
                            result.warnings
                        );
                    }

                    Self {
                        condition: Some(result.program),
                    }
                }
            },
            None => Self { condition: None },
        }
    }
}

#[async_trait::async_trait]
impl Plugin for DisableIntrospectionPlugin {
    async fn on_downstream_graphql_request(&self, ctx: &mut RequestExecutionContext) {
        if let Some(op) = &ctx.downstream_graphql_request {
            if op.is_introspection_query() {
                let should_disable = match &self.condition {
                    Some(program) => {
                        let downstream_http_req =
                            conductor_request_to_value(&ctx.downstream_http_request);
                        let mut target = TargetValue {
                            value: value!({}),
                            metadata: value!({
                              downstream_http_req: downstream_http_req,
                            }),
                            secrets: Secrets::default(),
                        };

                        match program.resolve(&mut Context::new(
                            &mut target,
                            ctx.vrl_shared_state(),
                            &TimeZone::default(),
                        )) {
                            Ok(ret) => match ret {
                                vrl::value::Value::Boolean(b) => b,
                                _ => {
                                    error!("DisableIntrospectionPlugin::vrl::condition must return a boolean, but returned a non-boolean value: {:?}, ignoring...", ret);

                                    true
                                }
                            },
                            Err(err) => {
                                error!("DisableIntrospectionPlugin::vrl::condition resolve error: {:?}", err);

                                ctx.short_circuit(
                                    GraphQLResponse::new_error("vrl runtime error")
                                        .into_with_status_code(StatusCode::BAD_GATEWAY),
                                );
                                return;
                            }
                        }
                    }
                    None => true,
                };

                if should_disable {
                    ctx.short_circuit(
                        GraphQLResponse::new_error("Introspection is disabled").into(),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        endpoint_runtime::EndpointRuntime, gateway::ConductorGateway,
        source::graphql_source::GraphQLSourceRuntime,
    };

    use super::*;
    use conductor_common::http::{ConductorHttpRequest, ConductorHttpResponse, HttpHeadersMap};
    use conductor_config::{plugins::VrlConfigReference, GraphQLSourceConfig};
    use http::Method;
    use httpmock::{Method::POST, MockServer};
    use serde_json::json;
    use tokio::test;

    async fn run(
        config: Option<DisableIntrospectionPluginConfig>,
        operation: String,
    ) -> ConductorHttpResponse {
        let mut plugins: Vec<Box<dyn crate::plugins::core::Plugin>> = vec![];

        if let Some(cfg) = config {
            plugins.push(Box::new(DisableIntrospectionPlugin::new(cfg)));
        }

        let http_mock = MockServer::start();
        http_mock.mock(|when, then| {
            when.method(POST).path("/graphql");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    &json!({
                        "data": {
                            "__typename": "Query"
                        },
                        "errors": null
                    })
                    .to_string(),
                );
        });

        let source = GraphQLSourceRuntime::new(GraphQLSourceConfig {
            endpoint: http_mock.url("/graphql"),
        });

        let mut headers = HttpHeadersMap::new();
        headers.append("bypass-introspection", "1".parse().unwrap());
        let request = ConductorHttpRequest {
            method: Method::POST,
            query_string: "".to_string(),
            uri: "/graphql".to_string(),
            body: json!({
                "query": operation,
                "variables": {}
            })
            .to_string()
            .into(),
            headers,
        };

        ConductorGateway::execute_test(EndpointRuntime::dummy(), Arc::new(source), plugins, request)
            .await
    }

    static INTROSPECTION_QUERY: &str = r#"
    query IntrospectionQuery {
        __schema {
          queryType { name }
          mutationType { name }
          subscriptionType { name }
          types {
            ...FullType
          }
          directives {
            name
            description
            locations
            args {
              ...InputValue
            }
          }
        }
      }
  
      fragment FullType on __Type {
        kind
        name
        description
        
        fields(includeDeprecated: true) {
          name
          description
          args {
            ...InputValue
          }
          type {
            ...TypeRef
          }
          isDeprecated
          deprecationReason
        }
        inputFields {
          ...InputValue
        }
        interfaces {
          ...TypeRef
        }
        enumValues(includeDeprecated: true) {
          name
          description
          isDeprecated
          deprecationReason
        }
        possibleTypes {
          ...TypeRef
        }
      }
  
      fragment InputValue on __InputValue {
        name
        description
        type { ...TypeRef }
        defaultValue
      }
  
      fragment TypeRef on __Type {
        kind
        name
        ofType {
          kind
          name
          ofType {
            kind
            name
            ofType {
              kind
              name
              ofType {
                kind
                name
                ofType {
                  kind
                  name
                  ofType {
                    kind
                    name
                    ofType {
                      kind
                      name
                      ofType {
                        kind
                        name
                        ofType {
                          kind
                          name
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }  
    "#;

    #[test]
    async fn should_allow_introspection_without_plugin() {
        let response = run(None, INTROSPECTION_QUERY.to_string()).await;
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}");
    }

    #[test]
    async fn disable_full_introspection_query() {
        let response = run(Some(Default::default()), INTROSPECTION_QUERY.to_string()).await;
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body,
            "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
        );
    }

    #[test]
    async fn disable_minimal_introspection_query() {
        let response = run(
            Some(Default::default()),
            "query { __schema { queryType { name }}}".to_string(),
        )
        .await;
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body,
            "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
        );
    }

    #[test]
    async fn disable_minimal_introspection_query_type_field() {
        let response = run(
            Some(Default::default()),
            "query { __type { name }}".to_string(),
        )
        .await;
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body,
            "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
        );
    }

    #[test]
    async fn disable_minimal_introspection_query_alias() {
        let response = run(
            Some(Default::default()),
            "query { s: __schema { queryType { name }}}".to_string(),
        )
        .await;
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body,
            "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
        );
    }

    #[test]
    async fn disable_type_name_only() {
        let response = run(Some(Default::default()), "query { __typename }".to_string()).await;
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body,
            "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
        );
    }

    #[test]
    async fn disable_typename_only_aliased() {
        let response = run(
            Some(Default::default()),
            "query { f: __typename }".to_string(),
        )
        .await;
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body,
            "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
        );
    }

    #[test]
    async fn allow_mixed_typename_and_fields() {
        let response = run(
            Some(Default::default()),
            "query { __typename id }".to_string(),
        )
        .await;
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}");
    }

    #[test]
    async fn disallow_mixed_schema_and_fields() {
        let response = run(
            Some(Default::default()),
            "query { field __schema { queryType { name }}}".to_string(),
        )
        .await;
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body,
            "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
        );
    }

    #[test]
    async fn should_allow_to_provide_simple_condition() {
        let response = run(
            Some(DisableIntrospectionPluginConfig {
                condition: Some(VrlConfigReference::Inline {
                    content: "true".to_string(),
                }),
            }),
            "query { field __schema { queryType { name }}}".to_string(),
        )
        .await;
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body,
            "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
        );
    }

    #[test]
    async fn should_allow_to_provide_complex_condition() {
        let response = run(
            Some(DisableIntrospectionPluginConfig {
                condition: Some(VrlConfigReference::Inline {
                    content: "%downstream_http_req.method == \"POST\"".to_string(),
                }),
            }),
            "query { field __schema { queryType { name }}}".to_string(),
        )
        .await;
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body,
            "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
        );
    }

    #[test]
    async fn should_allow_condition_to_bypass() {
        let response = run(
            Some(DisableIntrospectionPluginConfig {
                condition: Some(VrlConfigReference::Inline {
                    content: "%downstream_http_req.headers.\"bypass-introspection\" != \"1\""
                        .to_string(),
                }),
            }),
            "query { field __schema { queryType { name }}}".to_string(),
        )
        .await;
        assert_eq!(response.status, 200);
        assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}");
    }
}
