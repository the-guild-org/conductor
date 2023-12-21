use conductor_common::serde_utils::{
    JsonSchemaExample, JsonSchemaExampleMetadata, JsonSchemaExampleWrapperType,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The `http_get` plugin allows you to expose your GraphQL API over HTTP `GET` requests. This feature is fully compliant with the [GraphQL over HTTP specification](https://graphql.github.io/graphql-over-http/).
///
/// By enabling this plugin, you can execute GraphQL queries and mutations over HTTP `GET` requests, using HTTP query parameters, for example:
///
/// `GET /graphql?query=query%20%7B%20__typename%20%7D`
///
/// ### Query Parameters
///
/// For complete documentation of the supported query parameters, see the [GraphQL over HTTP specification](https://graphql.github.io/graphql-over-http/draft/#sec-GET).
///
/// - `query`: The GraphQL query to execute
///
/// - `variables` (optional): A JSON-encoded string containing the GraphQL variables
///
/// - `operationName` (optional): The name of the GraphQL operation to execute
///
/// ### Headers
///
/// To execute GraphQL queries over HTTP `GET` requests, you must set the `Content-Type` header to `application/json`, **or** the `Accept` header to `application/x-www-form-urlencoded` / `application/graphql-response+json`.
///
#[derive(Deserialize, Serialize, Debug, Clone, Default, JsonSchema)]
#[schemars(example = "http_get_example_1")]
#[schemars(example = "http_get_example_2")]
pub struct HttpGetPluginConfig {
    /// Allow mutations over GET requests.
    ///
    /// **The option is disabled by default:** this restriction is necessary to conform with the long-established semantics of safe methods within HTTP.
    #[serde(
        default = "mutations_default_value",
        skip_serializing_if = "Option::is_none"
    )]
    pub mutations: Option<bool>,
}

fn http_get_example_1() -> JsonSchemaExample<HttpGetPluginConfig> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new("Simple", None),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
            name: "http_get".to_string(),
        }),
        example: HttpGetPluginConfig { mutations: None },
    }
}

fn http_get_example_2() -> JsonSchemaExample<HttpGetPluginConfig> {
    JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Enable Mutations",
            Some("This example enables mutations over HTTP GET requests."),
        ),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
            name: "http_get".to_string(),
        }),
        example: HttpGetPluginConfig {
            mutations: Some(true),
        },
    }
}

fn mutations_default_value() -> Option<bool> {
    Some(false)
}
