use conductor_common::serde_utils::{
  JsonSchemaExample, JsonSchemaExampleMetadata, JsonSchemaExampleWrapperType,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The `cors` plugin enables [Cross-Origin Resource Sharing (CORS)](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) configuration for your GraphQL API.
///
/// By using this plugin, you can define rules for allowing cross-origin requests to your GraphQL server. This is essential for web applications that need to interact with your API from different domains.
///
#[derive(Deserialize, Serialize, Debug, Clone, JsonSchema)]
#[schemars(example = "cors_plugin_example1")]
#[schemars(example = "cors_plugin_example2")]
#[schemars(example = "cors_plugin_example3")]
pub struct CorsPluginConfig {
  /// `Access-Control-Allow-Credentials`: Specifies whether to include credentials in the CORS headers. Credentials can include cookies, authorization headers, or TLS client certificates.
  /// Indicates whether the response to the request can be exposed when the credentials flag is true.
  #[serde(
    default = "default_boolean_false",
    skip_serializing_if = "Option::is_none"
  )]
  pub allow_credentials: Option<bool>,

  /// `Access-Control-Allow-Methods`: Defines the HTTP methods allowed when accessing the resource. This is used in response to a CORS preflight request.
  /// Specifies the method or methods allowed when accessing the resource in response to a preflight request.
  /// You can also specify a special value "*" to allow any HTTP method to access the resource.
  #[serde(default = "default_wildcard", skip_serializing_if = "Option::is_none")]
  pub allowed_methods: Option<String>,

  /// `Access-Control-Allow-Origin`: Determines which origins are allowed to access the resource. It can be a specific origin or a wildcard for allowing any origin.
  /// You can also specify a special value "*" to allow any origin to access the resource.
  /// You can also specify a special value "reflect" to allow the origin of the incoming request to access the resource.
  #[serde(default = "default_wildcard", skip_serializing_if = "Option::is_none")]
  pub allowed_origin: Option<String>,

  /// `Access-Control-Allow-Headers`: Lists the headers allowed in actual requests. This helps in specifying which headers can be used when making the actual request.
  /// Used in response to a preflight request to indicate which HTTP headers can be used when making the actual request.
  /// You can also specify a special value "*" to allow any headers to be used when making the actual request, and the `Access-Control-Request-Headers` will be used from the incoming request.
  #[serde(default = "default_wildcard", skip_serializing_if = "Option::is_none")]
  pub allowed_headers: Option<String>,

  /// `Access-Control-Expose-Headers`: The "Access-Control-Expose-Headers" response header allows a server to indicate which response headers should be made available to scripts running in the browser, in response to a cross-origin request.
  /// You can also specify a special value "*" to allow any headers to be exposed to scripts running in the browser.
  #[serde(default = "default_wildcard", skip_serializing_if = "Option::is_none")]
  pub exposed_headers: Option<String>,

  /// `Access-Control-Allow-Private-Network`: Indicates whether requests from private networks are allowed when originating from public networks.
  #[serde(
    default = "default_boolean_false",
    skip_serializing_if = "Option::is_none"
  )]
  pub allow_private_network: Option<bool>,

  /// `Access-Control-Max-Age`: Indicates how long the results of a preflight request can be cached.
  /// This field represents the duration in seconds.
  #[serde(default = "defualt_max_age", skip_serializing_if = "Option::is_none")]
  pub max_age: Option<u64>,
}

impl Default for CorsPluginConfig {
  fn default() -> Self {
    Self {
      allow_credentials: default_boolean_false(),
      allowed_methods: default_wildcard(),
      allowed_origin: default_wildcard(),
      allowed_headers: default_wildcard(),
      exposed_headers: default_wildcard(),
      allow_private_network: default_boolean_false(),
      max_age: defualt_max_age(),
    }
  }
}

fn default_wildcard() -> Option<String> {
  Some("*".to_string())
}

fn default_boolean_false() -> Option<bool> {
  Some(false)
}

fn defualt_max_age() -> Option<u64> {
  None
}

fn cors_plugin_example1() -> JsonSchemaExample<CorsPluginConfig> {
  JsonSchemaExample {
        metadata: JsonSchemaExampleMetadata::new(
            "Strict CORS",
            Some("This example demonstrates how to configure the CORS plugin with a strict list of methods, headers and origins."),
        ),
        wrapper: Some(JsonSchemaExampleWrapperType::Plugin { name: "cors".into() }),
        example: CorsPluginConfig {
                allow_credentials: Some(true),
                exposed_headers: None,
                allowed_methods: Some("GET, POST".into()),
                allowed_origin: Some("https://example.com".into()),
                allowed_headers: Some("Content-Type, Authorization".into()),
                allow_private_network: Some(false),
                max_age: Some(3600),
            },
    }
}

fn cors_plugin_example2() -> JsonSchemaExample<CorsPluginConfig> {
  JsonSchemaExample {
    metadata: JsonSchemaExampleMetadata::new(
      "Permissive CORS",
      Some("This example demonstrates how to configure the CORS plugin with a permissive setup."),
    ),
    wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
      name: "cors".into(),
    }),
    example: CorsPluginConfig {
      allow_credentials: Some(true),
      exposed_headers: Some("*".into()),
      allowed_methods: Some("*".into()),
      allowed_origin: Some("*".into()),
      allowed_headers: Some("*".into()),
      allow_private_network: Some(true),
      max_age: Some(3600),
    },
  }
}

fn cors_plugin_example3() -> JsonSchemaExample<CorsPluginConfig> {
  JsonSchemaExample {
    metadata: JsonSchemaExampleMetadata::new(
      "Reflect Origin",
      Some(
        "This example demonstrates how to configure the CORS plugin with a reflect Origin setup.",
      ),
    ),
    wrapper: Some(JsonSchemaExampleWrapperType::Plugin {
      name: "cors".into(),
    }),
    example: CorsPluginConfig {
      allow_credentials: Some(true),
      exposed_headers: Some("*".into()),
      allowed_methods: Some("GET, POST".into()),
      allowed_origin: Some("reflect".into()),
      allowed_headers: Some("*".into()),
      allow_private_network: Some(false),
      max_age: Some(3600),
    },
  }
}
