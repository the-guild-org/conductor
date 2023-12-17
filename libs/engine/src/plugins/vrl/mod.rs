pub mod downstream_graphql_request;
pub mod downstream_http_request;
pub mod downstream_http_response;
pub mod plugin;
pub mod upstream_http_request;
pub mod utils;
pub mod vrl_functions;

#[cfg(test)]
mod tests {
    use httpmock::prelude::*;
    use serde_json::json;
    use std::sync::Arc;

    use conductor_common::{
        graphql::{GraphQLRequest, ParsedGraphQLRequest},
        http::{ConductorHttpRequest, ConductorHttpResponse, HeaderValue, Method, StatusCode},
    };
    use conductor_config::{
        plugins::{VrlConfigReference, VrlPluginConfig},
        GraphQLSourceConfig,
    };
    use reqwest::header::HeaderMap;
    use tokio::test;

    use crate::{
        endpoint_runtime::EndpointRuntime,
        gateway::ConductorGateway,
        plugins::{core::Plugin, vrl::plugin::VrlPlugin},
        request_execution_context::RequestExecutionContext,
        source::graphql_source::GraphQLSourceRuntime,
    };

    #[test]
    #[tracing_test::traced_test]
    async fn complete_flow_with_shared_state() {
        let plugin = VrlPlugin::new(VrlPluginConfig {
            on_downstream_http_request: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        authorization_header = %downstream_http_req.headers.authorization
                    "#,
                ),
            }),
            on_downstream_graphql_request: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        test = join!(["incoming query:", authorization_header])
                        log(test, level:"info")
                    "#,
                ),
            }),
            on_upstream_http_request: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        .upstream_http_req.headers."x-authorization" = "test-value"
                    "#,
                ),
            }),
            on_downstream_http_response: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        .downstream_http_res.headers."dotan-test" = "oopsi"
                        
                    "#,
                ),
            }),
        });

        let mut header_map: HeaderMap = HeaderMap::default();
        header_map.append("content-type", HeaderValue::from_static("application/json"));
        header_map.append(
            "authorization",
            HeaderValue::from_static("application/json"),
        );
        let request: ConductorHttpRequest = ConductorHttpRequest {
            body: "{\"query\": \"query { __typename }\"}".into(),
            uri: String::from("/graphql"),
            query_string: String::from(""),
            method: Method::POST,
            headers: header_map,
        };

        let http_mock = MockServer::start();
        let source_mock = http_mock.mock(|when, then| {
            when.method(POST)
                .path("/graphql")
                .header_exists("x-authorization");
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

        let response = ConductorGateway::execute_test(
            EndpointRuntime::dummy(),
            Arc::new(source),
            vec![Box::new(plugin)],
            request,
        )
        .await;
        source_mock.assert_hits(1);
        assert_eq!(response.status, StatusCode::OK);
        assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}");
        assert!(response
            .headers
            .get("dotan-test")
            .is_some_and(|v| v == "oopsi"));
    }

    #[test]
    #[should_panic]
    async fn test_ignore_invalid_vrl_compile() {
        VrlPlugin::new(VrlPluginConfig {
            on_downstream_http_request: Some(VrlConfigReference::Inline {
                // invalid because "b" doesn't exists
                content: String::from(
                    r#"
                        a = b
                    "#,
                ),
            }),
            on_downstream_graphql_request: None,
            on_upstream_http_request: None,
            on_downstream_http_response: None,
        });
    }

    #[test]
    async fn test_waterfall_of_hooks() {
        let mut header_map: HeaderMap = HeaderMap::default();
        header_map.append("content-type", HeaderValue::from_static("application/json"));
        let mut request: ConductorHttpRequest = ConductorHttpRequest {
            body: "{\"query\": \"query { __typename }\"}".into(),
            uri: String::from("/graphql"),
            query_string: String::from(""),
            method: Method::POST,
            headers: header_map,
        };

        let mut response = ConductorHttpResponse {
            body: "{\"data\": {\"__typename\": \"Query\"}}".into(),
            status: StatusCode::OK,
            headers: Default::default(),
        };

        let endpoint = EndpointRuntime::dummy();
        let mut ctx = RequestExecutionContext::new(&endpoint, request.clone());
        ctx.downstream_graphql_request = Some(
            ParsedGraphQLRequest::create_and_parse(GraphQLRequest {
                extensions: None,
                variables: None,
                operation: "query test { __typename }".to_string(),
                operation_name: Some("test".to_string()),
            })
            .unwrap(),
        );

        let plugin = VrlPlugin::new(VrlPluginConfig {
            on_downstream_http_request: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        var1 = "1"
                    "#,
                ),
            }),
            on_downstream_graphql_request: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        var2 = var1 + "2"
                    "#,
                ),
            }),
            on_upstream_http_request: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        var3 = var1 + var2 + "3"
                    "#,
                ),
            }),
            on_downstream_http_response: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        var4 = var1 + var2 + var3 + "4"
                        assert!(var4 == "11211234", message: "invalid value")
                    "#,
                ),
            }),
        });
        plugin.on_downstream_http_request(&mut ctx).await;
        plugin.on_downstream_graphql_request(&mut ctx).await;
        plugin
            .on_upstream_http_request(&mut ctx, &mut request)
            .await;
        plugin.on_downstream_http_response(&mut ctx, &mut response);
        assert!(!ctx.is_short_circuit());

        let plugin = VrlPlugin::new(VrlPluginConfig {
            on_downstream_http_request: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        var1 = "1"
                    "#,
                ),
            }),
            on_downstream_graphql_request: None,
            on_upstream_http_request: None,
            on_downstream_http_response: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        var2 = var1 + "2"
                        assert!(var2 == "12", message: "invalid value")
                    "#,
                ),
            }),
        });
        plugin.on_downstream_http_request(&mut ctx).await;
        plugin.on_downstream_graphql_request(&mut ctx).await;
        plugin
            .on_upstream_http_request(&mut ctx, &mut request)
            .await;
        plugin.on_downstream_http_response(&mut ctx, &mut response);
        assert!(!ctx.is_short_circuit());

        let plugin = VrlPlugin::new(VrlPluginConfig {
            on_downstream_http_request: None,
            on_downstream_graphql_request: None,
            on_upstream_http_request: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        var1 = "1"
                    "#,
                ),
            }),
            on_downstream_http_response: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        var2 = var1 + "2"
                        assert!(var2 == "12", message: "invalid value")
                    "#,
                ),
            }),
        });
        plugin.on_downstream_http_request(&mut ctx).await;
        plugin.on_downstream_graphql_request(&mut ctx).await;
        plugin
            .on_upstream_http_request(&mut ctx, &mut request)
            .await;
        plugin.on_downstream_http_response(&mut ctx, &mut response);
        assert!(!ctx.is_short_circuit());
    }

    #[test]
    async fn test_vrl_on_downstream_request_input_output() {
        let plugin = VrlPlugin::new(VrlPluginConfig {
            on_downstream_http_request: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        # input
                        assert!(%downstream_http_req.headers.authorization == "Bearer XYZ", message: "invalid value")
                        assert!(%downstream_http_req.headers."content-type" == "application/json", message: "invalid value")
                        assert!(%downstream_http_req.method == "POST", message: "invalid value")
                        assert!(%downstream_http_req.body == "{\"query\": \"query { __typename }\"}", message: "invalid value")
                        assert!(%downstream_http_req.uri == "/graphql", message: "invalid value")
                        assert!(%downstream_http_req.query_string == "test=1", message: "invalid value")

                        # output
                        .graphql.operation = "query override { __typename }"
                        .graphql.operation_name = "override"
                    "#,
                ),
            }),
            on_downstream_graphql_request: None,
            on_upstream_http_request: None,
            on_downstream_http_response: None,
        });

        let mut header_map: HeaderMap = HeaderMap::default();
        header_map.append("Authorization", HeaderValue::from_static("Bearer XYZ"));
        header_map.append("content-type", HeaderValue::from_static("application/json"));
        let request: ConductorHttpRequest = ConductorHttpRequest {
            body: "{\"query\": \"query { __typename }\"}".into(),
            uri: String::from("/graphql"),
            query_string: String::from("test=1"),
            method: Method::POST,
            headers: header_map,
        };

        let endpoint = EndpointRuntime::dummy();
        let mut ctx = RequestExecutionContext::new(&endpoint, request);

        plugin.on_downstream_http_request(&mut ctx).await;

        // In case of a VRL evaluation error, we should fail to short_circuit,
        // so it's safe to use assertion in VRL and check this condition here
        assert_eq!(ctx.short_circuit_response.is_none(), true);

        assert_eq!(ctx.downstream_graphql_request.is_some(), true);
        assert_eq!(
            ctx.downstream_graphql_request
                .as_ref()
                .unwrap()
                .request
                .operation_name,
            Some(String::from("override"))
        );
        assert_eq!(
            ctx.downstream_graphql_request
                .as_ref()
                .map(|v| v.request.operation.clone()),
            Some(String::from("query override { __typename }"))
        );
    }

    #[test]
    async fn test_vrl_on_downstream_graphql_request_input_output() {
        let plugin = VrlPlugin::new(VrlPluginConfig {
            on_downstream_http_request: None,
            on_downstream_graphql_request: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        # input
                        assert!(%downstream_graphql_req.operation == "query test { __typename }", message: "invalid value")

                        # output
                        .graphql.operation = "query override { __typename }"
                        .graphql.operation_name = "override"
                        .graphql.variables = {"test": 1, "works": true}
                        .graphql.extensions = {"metadata": "test"}
                    "#,
                ),
            }),
            on_upstream_http_request: None,
            on_downstream_http_response: None,
        });

        let mut header_map: HeaderMap = HeaderMap::default();
        header_map.append("Authorization", HeaderValue::from_static("Bearer XYZ"));
        header_map.append("content-type", HeaderValue::from_static("application/json"));
        let request: ConductorHttpRequest = ConductorHttpRequest {
            body: "{\"query\": \"query test { __typename }\"}".into(),
            uri: String::from("/graphql"),
            query_string: String::from(""),
            method: Method::POST,
            headers: header_map,
        };

        let endpoint = EndpointRuntime::dummy();
        let mut ctx = RequestExecutionContext::new(&endpoint, request);
        ctx.downstream_graphql_request = Some(
            ParsedGraphQLRequest::create_and_parse(GraphQLRequest {
                extensions: None,
                variables: None,
                operation: "query test { __typename }".to_string(),
                operation_name: Some("test".to_string()),
            })
            .unwrap(),
        );
        plugin.on_downstream_graphql_request(&mut ctx).await;

        // In case of a VRL evaluation error, we should fail to short_circuit,
        // so it's safe to use assertion in VRL and check this condition here
        assert_eq!(ctx.short_circuit_response.is_none(), true);
        assert_eq!(ctx.downstream_graphql_request.is_some(), true);
        assert_eq!(
            ctx.downstream_graphql_request
                .as_ref()
                .unwrap()
                .request
                .operation_name,
            Some(String::from("override"))
        );
        assert_eq!(
            ctx.downstream_graphql_request
                .as_ref()
                .map(|v| v.request.operation.clone()),
            Some(String::from("query override { __typename }"))
        );
        assert_eq!(
            ctx.downstream_graphql_request
                .as_ref()
                .and_then(|v| v.request.variables.clone()),
            Some(
                serde_json::json!({"test": 1, "works": true})
                    .as_object()
                    .cloned()
                    .unwrap()
            )
        );
        assert_eq!(
            ctx.downstream_graphql_request
                .as_ref()
                .and_then(|v| v.request.extensions.clone()),
            Some(
                serde_json::json!({"metadata": "test"})
                    .as_object()
                    .cloned()
                    .unwrap()
            )
        );
    }

    #[test]
    #[tracing_test::traced_test]
    async fn on_downstream_http_response_input_output() {
        let plugin = VrlPlugin::new(VrlPluginConfig {
            on_downstream_http_response: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                        # input
                        assert!(length(keys!(%downstream_http_res.headers)) == 0, message: "invalid header count")
                        assert!(%downstream_http_res.body == "{\"data\": {\"__typename\": \"Query\"}}", message: "invalid body value")
                        assert!(%downstream_http_res.status == 200, message: "invalid status value")

                        # output
                        .downstream_http_res.body = "override"
                        .downstream_http_res.status = 400
                        .downstream_http_res.headers."x-test" = "test"
                    "#,
                ),
            }),
            on_downstream_graphql_request: None,
            on_upstream_http_request: None,
            on_downstream_http_request: None,
        });

        let mut header_map: HeaderMap = HeaderMap::default();
        header_map.append("Authorization", HeaderValue::from_static("Bearer XYZ"));
        header_map.append("content-type", HeaderValue::from_static("application/json"));
        let request: ConductorHttpRequest = ConductorHttpRequest {
            body: "{\"query\": \"query { __typename }\"}".into(),
            uri: String::from("/graphql"),
            query_string: String::from("test=1"),
            method: Method::POST,
            headers: header_map,
        };

        let endpoint = EndpointRuntime::dummy();
        let mut ctx = RequestExecutionContext::new(&endpoint, request);
        let mut response = ConductorHttpResponse {
            body: "{\"data\": {\"__typename\": \"Query\"}}".into(),
            status: StatusCode::OK,
            headers: Default::default(),
        };

        plugin.on_downstream_http_response(&mut ctx, &mut response);

        assert_eq!(ctx.short_circuit_response.is_none(), true);
        assert!(response.body == "override");
        assert!(response.status == 400);
        assert!(response.headers.get("x-test").is_some_and(|v| v == "test"));
    }

    #[test]
    async fn on_upstream_http_request_input_output() {
        let plugin = VrlPlugin::new(VrlPluginConfig {
            on_upstream_http_request: Some(VrlConfigReference::Inline {
                content: String::from(
                    r#"
                    # input
                    

                    # output
                    .upstream_http_req.headers."x-test" = "test"
                    .upstream_http_req.headers."x-authorization" = "test-value"
                    .upstream_http_req.method = "PATCH"
                    .upstream_http_req.uri = "/upstream_else/graphql"
                    .upstream_http_req.query_string = "u=1"
                    .upstream_http_req.body = "{\"query\": \"query { override: __typename }\"}"
                "#,
                ),
            }),
            on_downstream_graphql_request: None,
            on_downstream_http_response: None,
            on_downstream_http_request: None,
        });

        let mut header_map: HeaderMap = HeaderMap::default();
        header_map.append("Authorization", HeaderValue::from_static("Bearer XYZ"));
        header_map.append("content-type", HeaderValue::from_static("application/json"));
        let request: ConductorHttpRequest = ConductorHttpRequest {
            body: "{\"query\": \"query { __typename }\"}".into(),
            uri: String::from("/graphql"),
            query_string: String::from("test=1"),
            method: Method::POST,
            headers: header_map,
        };

        let endpoint = EndpointRuntime::dummy();
        let mut ctx = RequestExecutionContext::new(&endpoint, request);
        let mut request = ConductorHttpRequest {
            body: "{\"query\": \"query { __typename }\"}".into(),
            uri: String::from("/upstream/graphql"),
            query_string: String::from(""),
            method: Method::POST,
            headers: Default::default(),
        };

        plugin
            .on_upstream_http_request(&mut ctx, &mut request)
            .await;

        assert!(ctx.short_circuit_response.is_none());
        assert_eq!(request.method, Method::PATCH);
        assert_eq!(request.uri, "/upstream_else/graphql");
        assert!(request.body == "{\"query\": \"query { override: __typename }\"}");
        assert_eq!(request.query_string, "u=1");
        assert!(request.headers.get("x-test").is_some_and(|v| v == "test"));
        assert!(request
            .headers
            .get("x-authorization")
            .is_some_and(|v| v == "test-value"));
    }
}
