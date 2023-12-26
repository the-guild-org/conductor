use conductor_common::{
  serde_utils::{JsonSchemaExample, JsonSchemaExampleMetadata, JsonSchemaExampleWrapperType},
  vrl_utils::VrlConfigReference,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[schemars(example = "disable_introspection_example1")]
#[schemars(example = "disable_introspection_example2")]
/// The `disable_introspection` plugin allows you to disable introspection for your GraphQL API.
///
/// A [GraphQL introspection query](https://graphql.org/learn/introspection/) is a special GraphQL query that returns information about the GraphQL schema of your API.
///
/// It it [recommended to disable introspection for production environments](https://escape.tech/blog/should-i-disable-introspection-in-graphql/), unless you have a specific use-case for it.
///
/// It can either disable introspection for all requests, or only for requests that match a specific condition (using VRL scripting language).
///
pub struct DisableIntrospectionPluginConfig {
  #[serde(default, skip_serializing_if = "Option::is_none")]
  /// A VRL condition that determines whether to disable introspection for the request. This condition is evaluated only if the incoming GraphQL request is detected as an introspection query.
  ///
  /// The condition is evaluated in the context of the incoming request and have access to the metadata field `%downstream_http_req` (fields: `body`, `uri`, `query_string`, `method`, `headers`).
  ///
  /// The condition must return a boolean value: return `true` to continue and disable the introspection, and `false` to allow the introspection to run.
  ///
  /// In case of a runtime error, or an unexpected return value, the script will be ignored and introspection will be disabled for the incoming request.
  pub condition: Option<VrlConfigReference>,
}

fn disable_introspection_example1() -> JsonSchemaExample<DisableIntrospectionPluginConfig> {
  JsonSchemaExample {
    metadata: JsonSchemaExampleMetadata::new(
      "Disable Introspection",
      Some("This example disables introspection for all requests for the configured Endpoint."),
    ),
    wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
      name: "disable_introspection".to_string(),
    }),
    example: DisableIntrospectionPluginConfig {
      ..Default::default()
    },
  }
}

fn disable_introspection_example2() -> JsonSchemaExample<DisableIntrospectionPluginConfig> {
  JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Conditional",
            Some(
                "This example disables introspection for all requests that doesn't have the \"bypass-introspection\" HTTP header.",
            ),
        ),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
            name: "disable_introspection".to_string(),
        }),
        example: DisableIntrospectionPluginConfig {
            condition: Some(VrlConfigReference::Inline { content: "%downstream_http_req.headers.\"bypass-introspection\" != \"1\"".to_string() }),
        },
    }
}
