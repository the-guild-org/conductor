use conductor_common::{
    graphql::GraphQLRequest,
    http::{ConductorHttpRequest, HttpHeadersMap, Method},
    vrl_utils::VrlConfigReference,
};
use e2e::suite::TestSuite;
use tokio::test;

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
    let test = TestSuite {
        plugins: vec![],
        ..Default::default()
    };
    let response = test
        .run_graphql_request(GraphQLRequest {
            operation: INTROSPECTION_QUERY.to_string(),
            ..Default::default()
        })
        .await;

    assert_eq!(response.status, 200);
    assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}");
}

#[test]
async fn disable_full_introspection_query() {
    let test = TestSuite {
        plugins: vec![Box::new(disable_introspection_plugin::Plugin::new(
            Default::default(),
        ))],
        ..Default::default()
    };
    let response = test
        .run_graphql_request(GraphQLRequest {
            operation: INTROSPECTION_QUERY.to_string(),
            ..Default::default()
        })
        .await;
    assert_eq!(response.status, 200);
    assert_eq!(
        response.body,
        "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
    );
}

#[test]
async fn disable_minimal_introspection_query() {
    let test = TestSuite {
        plugins: vec![Box::new(disable_introspection_plugin::Plugin::new(
            Default::default(),
        ))],
        ..Default::default()
    };
    let response = test
        .run_graphql_request(GraphQLRequest {
            operation: "query { __schema { queryType { name }}}".to_string(),
            ..Default::default()
        })
        .await;

    assert_eq!(response.status, 200);
    assert_eq!(
        response.body,
        "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
    );
}

#[test]
async fn disable_minimal_introspection_query_type_field() {
    let test = TestSuite {
        plugins: vec![Box::new(disable_introspection_plugin::Plugin::new(
            Default::default(),
        ))],
        ..Default::default()
    };
    let response = test
        .run_graphql_request(GraphQLRequest {
            operation: "query { __type { name }}".to_string(),
            ..Default::default()
        })
        .await;
    assert_eq!(response.status, 200);
    assert_eq!(
        response.body,
        "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
    );
}

#[test]
async fn disable_minimal_introspection_query_alias() {
    let test = TestSuite {
        plugins: vec![Box::new(disable_introspection_plugin::Plugin::new(
            Default::default(),
        ))],
        ..Default::default()
    };
    let response = test
        .run_graphql_request(GraphQLRequest {
            operation: "query { s: __schema { queryType { name }}}".to_string(),
            ..Default::default()
        })
        .await;

    assert_eq!(response.status, 200);
    assert_eq!(
        response.body,
        "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
    );
}

#[test]
async fn disable_type_name_only() {
    let test = TestSuite {
        plugins: vec![Box::new(disable_introspection_plugin::Plugin::new(
            Default::default(),
        ))],
        ..Default::default()
    };
    let response = test
        .run_graphql_request(GraphQLRequest {
            operation: "query { __typename }".to_string(),
            ..Default::default()
        })
        .await;

    assert_eq!(response.status, 200);
    assert_eq!(
        response.body,
        "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
    );
}

#[test]
async fn disable_typename_only_aliased() {
    let test = TestSuite {
        plugins: vec![Box::new(disable_introspection_plugin::Plugin::new(
            Default::default(),
        ))],
        ..Default::default()
    };
    let response = test
        .run_graphql_request(GraphQLRequest {
            operation: "query { f: __typename }".to_string(),
            ..Default::default()
        })
        .await;
    assert_eq!(response.status, 200);
    assert_eq!(
        response.body,
        "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
    );
}

#[test]
async fn allow_mixed_typename_and_fields() {
    let test = TestSuite {
        plugins: vec![Box::new(disable_introspection_plugin::Plugin::new(
            Default::default(),
        ))],
        ..Default::default()
    };
    let response = test
        .run_graphql_request(GraphQLRequest {
            operation: "query { __typename id }".to_string(),
            ..Default::default()
        })
        .await;
    assert_eq!(response.status, 200);
    assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}");
}

#[test]
async fn disallow_mixed_schema_and_fields() {
    let test = TestSuite {
        plugins: vec![Box::new(disable_introspection_plugin::Plugin::new(
            Default::default(),
        ))],
        ..Default::default()
    };
    let response = test
        .run_graphql_request(GraphQLRequest {
            operation: "query { field __schema { queryType { name }}}".to_string(),
            ..Default::default()
        })
        .await;

    assert_eq!(response.status, 200);
    assert_eq!(
        response.body,
        "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
    );
}

#[test]
async fn should_allow_to_provide_simple_condition() {
    let test = TestSuite {
        plugins: vec![Box::new(disable_introspection_plugin::Plugin::new(
            disable_introspection_plugin::Config {
                condition: Some(VrlConfigReference::Inline {
                    content: "true".to_string(),
                }),
            },
        ))],
        ..Default::default()
    };
    let response = test
        .run_graphql_request(GraphQLRequest {
            operation: "query { field __schema { queryType { name }}}".to_string(),
            ..Default::default()
        })
        .await;

    assert_eq!(response.status, 200);
    assert_eq!(
        response.body,
        "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
    );
}

#[test]
async fn should_allow_to_provide_complex_condition() {
    let test = TestSuite {
        plugins: vec![Box::new(disable_introspection_plugin::Plugin::new(
            disable_introspection_plugin::Config {
                condition: Some(VrlConfigReference::Inline {
                    content: "%downstream_http_req.method == \"POST\"".to_string(),
                }),
            },
        ))],
        ..Default::default()
    };
    let response = test
        .run_graphql_request(GraphQLRequest {
            operation: "query { field __schema { queryType { name }}}".to_string(),
            ..Default::default()
        })
        .await;

    assert_eq!(response.status, 200);
    assert_eq!(
        response.body,
        "{\"errors\":[{\"message\":\"Introspection is disabled\"}]}"
    );
}

#[test]
async fn should_allow_condition_to_bypass() {
    let test = TestSuite {
        plugins: vec![Box::new(disable_introspection_plugin::Plugin::new(
            disable_introspection_plugin::Config {
                condition: Some(VrlConfigReference::Inline {
                    content: "%downstream_http_req.headers.\"bypass-introspection\" != \"1\""
                        .to_string(),
                }),
            },
        ))],
        ..Default::default()
    };
    let mut req_headers = HttpHeadersMap::new();
    req_headers.append("bypass-introspection", "1".parse().unwrap());
    let response = test
        .run_http_request(ConductorHttpRequest {
            body: GraphQLRequest {
                operation: "query { field __schema { queryType { name }}}".to_string(),
                ..Default::default()
            }
            .to_string()
            .into(),
            method: Method::POST,
            headers: req_headers,
            ..Default::default()
        })
        .await;

    assert_eq!(response.status, 200);
    assert_eq!(response.body, "{\"data\":{\"__typename\":\"Query\"}}");
}
