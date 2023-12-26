use conductor_common::serde_utils::{
  JsonSchemaExample, JsonSchemaExampleMetadata, JsonSchemaExampleWrapperType,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default, JsonSchema)]
#[schemars(example = "graphiql_example")]
/// This plugin adds a GraphiQL interface to your Endpoint.
///
/// This plugin is rendering the GraphiQL interface for HTTP `GET` requests, that are not intercepted by other plugins.
pub struct GraphiQLPluginConfig {
  #[serde(
    default = "headers_editor_enabled_default_value",
    skip_serializing_if = "Option::is_none"
  )]
  /// Enable/disable the HTTP headers editor in the GraphiQL interface.
  pub headers_editor_enabled: Option<bool>,
}

fn graphiql_example() -> JsonSchemaExample<GraphiQLPluginConfig> {
  JsonSchemaExample {
    metadata: JsonSchemaExampleMetadata::new("Enable GraphiQL", None),
    wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
      name: "graphiql".to_string(),
    }),
    example: GraphiQLPluginConfig {
      headers_editor_enabled: Default::default(),
    },
  }
}

fn headers_editor_enabled_default_value() -> Option<bool> {
  Some(true)
}

// At some point, it might be worth supporting more options. see:
// https://github.com/dotansimha/graphql-yoga/blob/main/packages/graphiql/src/YogaGraphiQL.tsx#L35
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GraphiQLSource {
  pub endpoint: String,
  pub query: String,
  #[serde(rename = "isHeadersEditorEnabled")]
  pub headers_editor_enabled: bool,
}
